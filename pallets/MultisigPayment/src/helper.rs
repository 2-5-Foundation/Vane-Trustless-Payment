#![cfg_attr(not(feature ="std"),no_std)]

// Helper utilities.
// The creation of a multi_account_id should be internal and opaque to the outside world.
// Functionalities present
// 1. Deriving multi acccount id
// 2. Creation multi account id storage
//

use super::pallet::*;
use frame_system::pallet_prelude::*;
use frame_support::pallet_prelude::*;
use sp_runtime::{
	MultiAddress,
	traits::{TrailingZeroInput}
};
use codec::{Encode, Decode};
use sp_std::mem::drop;

pub use utils::*;
pub mod utils{
	use frame_support::dispatch::RawOrigin;
	use frame_support::traits::{Currency, ExistenceRequirement};
	use sp_io::hashing::blake2_256;
	use sp_runtime::traits::StaticLookup;
	use super::*;
	use frame_system::{Account, AccountInfo};
	use frame_support::{
		dispatch::{
			DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo, GetDispatchInfo,
			PostDispatchInfo,
		},
	};
	use sp_runtime::{
		traits::{Dispatchable, TrailingZeroInput, Zero},
		DispatchError,
	};

	// A struct by which it should be used as a source of signatures.
	#[derive(Encode, Decode, Clone,PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AccountSigners<T: Config>{
		payee: T::AccountId,
		payer: T::AccountId,
		resolver: Option<Resolver<T>>,
	}


	// This will act as a dispute resolution methods. A user will have to choose which method
	// is the best for a given dispute which may arise.
	#[derive(Encode, Decode, Clone,PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub enum Resolver<T: Config>{
		// A legal team if chosen will be authorized to sign the transaction
		LegalTeam(T::AccountId),
		// A governance vote ( A Dao ) wil have to vote to favor which way the transaction
		// should be signed
		Governance,
		//some future time feature
		Both(T::AccountId)
	}


	// This should be used as a parameter for choosing which Resolving method should take place
	#[derive(Encode, Decode, Clone,PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum ResolverChoice{
		LegalTeam,
		Governance,
		None
	}


	impl<T> AccountSigners<T> where T:Config{
		pub fn new(
			payee: T::AccountId,
			payer: T::AccountId,
			resolver:Option<Resolver<T>>
		) -> Self{
			AccountSigners{
				payee,
				payer,
				resolver,
			}
		}
		pub(super) fn get_payer(&self) -> &T::AccountId{
			&self.payer
		}

		pub(super) fn get_payee(&self) -> &T::AccountId{
			&self.payee
		}

		pub(super) fn get_resolver(&self) -> &Option<Resolver<T>>{ &self.resolver }

		// refer here https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html?highlight=enum#enum-values
		pub(super) fn get_legal_account(&self) -> Option<&T::AccountId>{
			if let Some(Resolver::LegalTeam(account)) = &self.resolver{
				Some(account)
			}else{
				None
			}
		}
	}

	// Call executed struct information
	#[derive(Encode, Decode, Clone,PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct CallExecuted<T:Config>{
		time: T::BlockNumber,
		payer: T::AccountId,
		payee: T::AccountId,
		allowed_multi_id: T::AccountId,
		confirmed_multi_id: T::AccountId,
		proof: T::Hash
	}

	impl<T> CallExecuted<T> where T: Config {
		pub(super) fn new(
			time: T::BlockNumber,
			payer: T::AccountId,
			payee: T::AccountId,
			allowed_multi_id: T::AccountId,
			confirmed_multi_id: T::AccountId,
			proof: T::Hash
		)-> Self{
			CallExecuted{
				time,
				payer,
				payee,
				allowed_multi_id,
				confirmed_multi_id,
				proof
			}

		}
	}


	// Revert Fund reasons enum
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum RevertReasons{
		// The fee will be refunded, the payer must show a proof of wrong address.
		WrongPayeeAddress,
		// We should introduce sort of punishment, This reason should be taken seriously and
		// at the moment it should be only used in non trade operation.
		ChangeOfDecision,
		// Seller's fault, this is when a resolver intervene
		PayeeMisbehaviour
	}

	// Seller's reason to make fund go through when a buyer misbehave
	#[derive(Encode, Decode, Clone,PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum PayeeReason {
		PayerMisbehaviour
	}

	// Confirmation enum which will be used to confirm the account_ids before dispatching multi-sig Call
	#[derive(Encode, Decode, Clone,PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum Confirm{
		Payer,
		Payee
	}



	impl<T:Config> Pallet<T>{

		// Call if there are all confirmed signers


		// Call if there is only 1 confirmed signer

		// Inner functionality for the opening of multi-sig account
		pub(crate) fn inner_vane_pay_wo_resolver(
			payer: T::AccountId,
			payee: T::AccountId,
			amount: BalanceOf<T>
		) -> DispatchResult{

			let accounts = AccountSigners::<T>::new(payee,payer.clone(),None);
			let multi_id = Self::derive_multi_id(accounts.clone());
			AllowedSigners::<T>::insert(&payer,accounts);
			Self::create_multi_account(multi_id.clone())?;

			let time = <frame_system::Pallet::<T>>::block_number();

			Self::deposit_event(Event::MultiAccountCreated{
				account_id: multi_id.clone(),
				timestamp: time
			});

			// Transfer balance from Payer to Multi_Id
			T::Currency::transfer(&payer,&multi_id,amount, ExistenceRequirement::KeepAlive)?;

			Self::deposit_event(Event::BalanceTransferredAndLocked {
				to_multi_id: multi_id,
				from: payer,
				timestamp: time
			});

			Ok(())
		}

		// Dispatching Call helper
		pub(crate) fn dispatch_transfer_call(
			//proof:T::Hash,
			//payer: T::AccountId,
			payee: AccountFor<T>,
			allowed_multi_id: T::AccountId,
			confirmed_multi_id: T::AccountId
		) -> DispatchResult{
			// Store the proof and associated data of call execution
			// construct transfer call from pallet balances
			let multi_account_id = <<T as frame_system::Config>::Lookup as StaticLookup>
			::unlookup(confirmed_multi_id.clone());

			//let call = pallet_balances::Call::transfer_all::<T,()>{dest: multi_account_id, keep_alive: false};
			//let result = call.dispatch(RawOrigin::Signed(allowed_multi_id.clone()).into());
			<pallet_balances::Pallet<T,()>>::transfer_all(RawOrigin::Signed(allowed_multi_id).into(),payee,false)?;

			Ok(())
		}


		// Takes in a multi_id account and register it to Account storage in system pallet

		pub(crate) fn create_multi_account(multi_id: T::AccountId) -> DispatchResult{
			let account_info = AccountInfo::<T::Index, T::AccountData>{
				consumers:1,
				..Default::default()
			};

			// Ensure the multi_id account is not yet registered in the storage
			ensure!(!<frame_system::Pallet<T>>::account_exists(&multi_id),Error::<T>::MultiAccountExists);

			// Register to frame_system Account Storage item;
			<frame_system::Account<T>>::set(multi_id,account_info);
			Ok(())
		}


		// Now , we are only focusing legal team Resolver variant in multi_id generation
		// We can do better on this function definition
		pub(crate) fn derive_multi_id(account_object: AccountSigners<T>) -> T::AccountId{

				let (acc1, acc2, opt_acc3) =
					match account_object.get_resolver(){
					Some(resolver) => {
						 (account_object.get_payee(),account_object.get_payer(),account_object.get_legal_account())
					},
					None => (account_object.get_payee(),account_object.get_payer(),None)
				};

			let multi_account = if let Some(acc3) = opt_acc3{
				let entropy = (b"vane/salt",acc1,acc2,acc3).using_encoded(blake2_256);
				 Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
					.expect("infinite length input; no invalid inputs for type; qed")
			}else{
				let entropy = (b"vane/salt",acc1,acc2).using_encoded(blake2_256);
				Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
					.expect("infinite length input; no invalid inputs for type; qed")
			};

			multi_account

		}
	}


}
