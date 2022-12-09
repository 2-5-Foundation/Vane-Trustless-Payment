#![cfg_attr(not(feature = "std"), no_std)]

// Helper utilities.
// The creation of a multi_account_id should be internal and opaque to the outside world.
// Functionalities present
// 1. Deriving multi account id
// 2. Creation multi account id storage
// 3. Defining of AccountSigner's object
// 4. Defining AccountResolver's enum
// 5.
//

use super::pallet::*;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::{traits::TrailingZeroInput, MultiAddress};
use sp_std::mem::drop;

pub use utils::*;
pub mod utils {
	use super::*;
	use frame_support::{
		dispatch::{
			DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo, GetDispatchInfo,
			PostDispatchInfo, RawOrigin,
		},
		traits::{Currency, ExistenceRequirement},
	};
	use frame_system::{Account, AccountInfo};
	use sp_io::hashing::blake2_256;
	use sp_runtime::{
		traits::{Dispatchable, StaticLookup, TrailingZeroInput, Zero},
		DispatchError,
	};

	// A struct by which it should be used as a source of signatures.
	#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AccountSigners<T: Config> {
		payee: T::AccountId,
		payer: T::AccountId,
		resolver: Option<Resolver<T>>,
	}

	// This will act as a dispute resolution methods. A user will have to choose which method
	// is the best for a given dispute which may arise.
	#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub enum Resolver<T: Config> {
		// A legal team if chosen will be authorized to sign the transaction
		LegalTeam(T::AccountId),
		// A governance vote ( A Dao ) wil have to vote to favor which way the transaction
		// should be signed
		Governance,
		//some future time feature
		Both(T::AccountId),
	}

	// This should be used as a parameter for choosing which Resolving method should take place
	#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum ResolverChoice {
		LegalTeam,
		Governance,
		None,
	}

	impl<T> AccountSigners<T>
	where
		T: Config,
	{
		pub fn new(
			payee: T::AccountId,
			payer: T::AccountId,
			resolver: Option<Resolver<T>>,
		) -> Self {
			AccountSigners { payee, payer, resolver }
		}
		pub(super) fn get_payer(&self) -> &T::AccountId {
			&self.payer
		}

		pub(super) fn get_payee(&self) -> &T::AccountId {
			&self.payee
		}

		pub(super) fn get_resolver(&self) -> &Option<Resolver<T>> {
			&self.resolver
		}

		// refer here https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html?highlight=enum#enum-values
		pub(super) fn get_legal_account(&self) -> Option<&T::AccountId> {
			if let Some(Resolver::LegalTeam(account)) = &self.resolver {
				Some(account)
			} else {
				None
			}
		}
	}

	// Call executed struct information
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct CallExecuted<T: Config> {
		payer: T::AccountId,
		payee: T::AccountId,
		allowed_multi_id: T::AccountId,
		confirmed_multi_id: T::AccountId,
		proof: T::Hash,
		time: T::BlockNumber,
	}

	impl<T> CallExecuted<T>
	where
		T: Config,
	{
		pub(super) fn new(
			payer: T::AccountId,
			payee: T::AccountId,
			allowed_multi_id: T::AccountId,
			confirmed_multi_id: T::AccountId,
			proof: T::Hash,
			time: T::BlockNumber,
		) -> Self {
			CallExecuted { payer, payee, allowed_multi_id, confirmed_multi_id, proof, time }
		}
	}

	// Revert Fund reasons enum
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum RevertReasons {
		// The fee will be refunded, the payer must show a proof of wrong address.
		WrongPayeeAddress,
		// We should introduce sort of punishment, This reason should be taken seriously and
		// at the moment it should be only used in non trade operation.
		ChangeOfDecision,
		// Seller's fault, this is when a resolver intervene
		PayeeMisbehaviour,
	}

	// Seller's reason to make fund go through when a buyer misbehave
	#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum PayeeReason {
		PayerMisbehaviour,
	}

	// Confirmation enum which will be used to confirm the account_ids before dispatching multi-sig
	// Call
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum Confirm {
		Payer,
		Payee,
	}

	impl<T: Config> Pallet<T> {
		// Call if there are all confirmed signers

		// Call if there is only 1 confirmed signer

		// Inner functionality for the opening of multi-sig account
		pub(crate) fn inner_vane_pay_wo_resolver(
			payer: T::AccountId,
			payee: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let accounts = AccountSigners::<T>::new(payee, payer.clone(), None);
			let multi_id = Self::derive_multi_id(accounts.clone());
			AllowedSigners::<T>::insert(&payer, accounts);
			Self::create_multi_account(multi_id.clone())?;

			let time = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::MultiAccountCreated {
				account_id: multi_id.clone(),
				timestamp: time,
			});

			// Transfer balance from Payer to Multi_Id
			T::Currency::transfer(&payer, &multi_id, amount, ExistenceRequirement::KeepAlive)?;

			Self::deposit_event(Event::BalanceTransferredAndLocked {
				to_multi_id: multi_id,
				from: payer,
				timestamp: time,
			});

			Ok(())
		}

		// Dispatching Call helper
		pub(crate) fn dispatch_transfer_call(
			proof: T::Hash,
			payer: T::AccountId,
			payee: T::AccountId,
			allowed_multi_id: T::AccountId,
			confirmed_multi_id: T::AccountId,
		) -> DispatchResult {
			// Store the proof and associated data of call execution
			// construct transfer call from pallet balances
			let acc_payee = payee.clone();
			let payee = <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(payee);

			<pallet_balances::Pallet<T, ()>>::transfer_all(
				RawOrigin::Signed(allowed_multi_id.clone()).into(),
				payee,
				false,
			)
			.map_err(|_| Error::<T>::MultiSigCallFailed)?;

			let time = <frame_system::Pallet<T>>::block_number();

			let call_exe_object = CallExecuted::<T>::new(
				payer.clone(),
				acc_payee,
				allowed_multi_id,
				confirmed_multi_id.clone(),
				proof,
				time,
			);

			AccountMultiTxns::<T>::mutate(payer, |vec| vec.push(call_exe_object));

			Self::deposit_event(Event::CallExecuted {
				multi_id: confirmed_multi_id,
				timestamp: time,
			});

			Ok(())
		}

		// Takes in a multi_id account and register it to Account storage in system pallet

		pub(crate) fn create_multi_account(multi_id: T::AccountId) -> DispatchResult {
			let account_info =
				AccountInfo::<T::Index, T::AccountData> {..Default::default() };

			// Ensure the multi_id account is not yet registered in the storage
			// Vijay you must allow to re-use multi_id if its the same payer and payee.
			if <frame_system::Pallet<T>>::account_exists(&multi_id){
				return Ok(())
			}else{
				// Register to frame_system Account Storage item;
				<frame_system::Account<T>>::set(multi_id, account_info);
				Ok(())
			}

		}

		// Now , we are only focusing legal team Resolver variant in multi_id generation
		// We can do better on this function definition
		pub(crate) fn derive_multi_id(account_object: AccountSigners<T>) -> T::AccountId {
			let (acc1, acc2, opt_acc3) = match account_object.get_resolver() {
				Some(resolver) => (
					account_object.get_payee(),
					account_object.get_payer(),
					account_object.get_legal_account(),
				),
				None => (account_object.get_payee(), account_object.get_payer(), None),
			};

			let multi_account = if let Some(acc3) = opt_acc3 {
				let entropy = (b"vane/salt", acc1, acc2, acc3).using_encoded(blake2_256);
				Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
					.expect("infinite length input; no invalid inputs for type; qed")
			} else {
				let entropy = (b"vane/salt", acc1, acc2).using_encoded(blake2_256);
				Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
					.expect("infinite length input; no invalid inputs for type; qed")
			};

			multi_account
		}
	}
}
