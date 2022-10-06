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
		buyer: T::AccountId,
		seller: T::AccountId,
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

	impl<T> AccountSigners<T> where T:Config{
		pub(super) fn new(
			buyer: T::AccountId,
			seller: T::AccountId,
			resolver:Option<Resolver<T>>
		) -> Self{
			AccountSigners{
				buyer,
				seller,
				resolver,
			}
		}
		pub(super) fn get_buyer(&self) -> &T::AccountId{
			&self.buyer
		}

		pub(super) fn get_seller(&self) -> &T::AccountId{
			&self.seller
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


	// A struct that should be used as a reference when a payee account is not provided.
	// Mostly to be used in a trade payment.
	#[derive(Encode, Decode, Clone,PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Order<T: Config>{
		order_number:u32,
		account: T::AccountId
	}

	// Revert Fund reasons enum
	#[derive(Encode, Decode, Clone,PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum RevertReasons{
		// The fee will be refunded
		WrongPayeeAddress,
		// We should introduce sort of punishment
		ChangeOfDecision
	}


	impl<T:Config> Pallet<T>{

		// Inner functionality for the opening of multisig account
		pub(crate) fn inner_operate(buyer: T::AccountId, multi_id: T::AccountId) -> DispatchResult{

			Ok(())
		}


		pub(crate) fn create_multi_account(multi_id: T::AccountId) -> DispatchResult{


			Ok(())
		}

		// Now , we are only focusing legal team Resolver variant in multi_id generation
		pub(crate) fn derive_multi_id(account_object: AccountSigners<T>) -> T::AccountId{

				let (acc1, acc2, opt_acc3) =
					match account_object.get_resolver(){
					Some(resolver) => {
						 (account_object.get_buyer(),account_object.get_seller(),account_object.get_legal_account())
					},
					None => (account_object.get_buyer(),account_object.get_seller(),None)
				};

			let entropy = (b"vane/salt",acc1,acc2,opt_acc3.unwrap()).using_encoded(blake2_256);
			Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
				.expect("infinite length input; no invalid inputs for type; qed")
		}
	}


}
