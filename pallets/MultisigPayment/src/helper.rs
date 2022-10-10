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

pub use utils::*;
pub mod utils{
	use sp_io::hashing::blake2_256;
	use sp_runtime::traits::StaticLookup;
	use super::*;
	use frame_system::AccountInfo;
	// A struct by which it should be used as a source of signatures.
	#[derive(Encode, Decode, Clone,PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AccountSigners<T: Config>{
		payer: T::AccountId,
		payee: T::AccountId,
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
	}


	impl<T> AccountSigners<T> where T:Config{
		pub(super) fn new(
			payer: T::AccountId,
			payee: T::AccountId,
			resolver:Option<Resolver<T>>
		) -> Self{
			AccountSigners{
				payer,
				payee,
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


	// Revert Fund reasons enum
	#[derive(Encode, Decode, Clone,PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
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
	#[derive(Encode, Decode, Clone,PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum Confirm{
		Payer,
		Payee,
	}



	impl<T:Config> Pallet<T>{

		// Inner functionality for the opening of multi-sig account
		pub(crate) fn inner_vane_pay_wo_resolver(
			payer: T::AccountId,
			payee: T::AccountId,

		) -> DispatchResult{

			let Accounts = AccountSigners::<T>::new(payer,payee,None);
			let multi_id = Self::derive_multi_id(Accounts);

			Ok(())
		}


		pub(crate) fn create_multi_account(multi_id: T::AccountId) -> DispatchResult{
			let AccountInfo = AccountInfo::<T::Index, T::AccountData>{
				consumers:1,
				..Default::default()
			};

			// Ensure the multi_id account is not yet registered in the storage
			ensure!(!<frame_system::Pallet<T>>::account_exists(&multi_id),Error::<T>::MultiAccountExists);

			// Register to frame_system Account Storage item;
			<frame_system::Account<T>>::set(multi_id,AccountInfo);
			Ok(())
		}

		// Now , we are only focusing legal team Resolver variant in multi_id generation
		pub(crate) fn derive_multi_id(account_object: AccountSigners<T>) -> T::AccountId{

				let (acc1, acc2, opt_acc3) =
					match account_object.get_resolver(){
					Some(resolver) => {
						 (account_object.get_payer(),account_object.get_payee(),account_object.get_legal_account())
					},
					None => (account_object.get_payer(),account_object.get_payee(),None)
				};

			let entropy = (b"vane/salt",acc1,acc2,opt_acc3.unwrap()).using_encoded(blake2_256);
			Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
				.expect("infinite length input; no invalid inputs for type; qed")
		}
	}


}