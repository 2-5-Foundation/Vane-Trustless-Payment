#![cfg_attr(not(feature ="std")no_std)]

use super::pallet::{self,*};

pub use utils::*;
pub mod utils{
	use super::*;

	// A struct by which it should be used as a source of signatures.
	#[derive(Encode, Decode, Clone, Copy, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub struct AccountSigners<T>{
		buyer: AccountFor<T>,
		seller: AccountFor<T>,
		resolver: Option<Resolver<T>>,
	}

	// This will act as a dispute resolution methods. A user will have to choose which method
	// is the best for a given dispute which may arise.
	#[derive(Encode, Decode, Clone, Copy, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum Resolver<T>{
		// A legal team if chosen will be authorized to sign the transaction
		legal_team(AccountFor<T>),
		// A governance vote ( A Dao ) wil have to vote to favor which way the transaction
		// should be signed
		governance,
		//some future time feature
		both(AccountFor<T>)
	}

	impl<T> AccountSigners<T> where T: Config{
		pub(super) fn new(
			buyer: AccountFor<T>,
			seller:AccountFor<T>,
			resolver:Option<Resolver<T>>
		) -> Self{
			AccountSigners{
				buyer,
				seller,
				resolver,
			}
		}
		pub(super) fn get_buyer(&self) -> AccountFor<T>{
			self.buyer
		}

		pub(super) fn get_seller(&self) -> AccountFor<T>{
			self.seller
		}

		pub(super) fn get_resolver(self) -> Option<Resolver<T>>{ self.resolver }

		// refer here https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html?highlight=enum#enum-values
		pub(super) fn get_legal_account(&self) -> Option<AccountFor<T>>{
			if let Resolver::legal_team(account) = self.resolver{
				Some(*account)
			}else{
				None
			}
		}
	}


	// A struct that should be used as a reference when a payee account is not provided.
	// Mostly to be used in a trade payment.
	#[derive(Encode, Decode, Clone, Copy, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub struct Order<T>{
		order_number:u32,
		account: AccountFor<T>
	}


	impl<T:Config> Pallet<T>{

		// Now , we are only focusing legal team Resolver variant in multi_id generation
		fn derive_multi_id(account_object: AccountSigners<T>) -> T::AccountId{
			if is_resolver{
				let (acc1, acc2, opt_acc3) = match account_object.get_resolver(){
					Some(resolver) => {
						return (account_object.get_buyer(),account_object.get_seller(),account_object.get_legal_account())
					},
					None => (account_object.get_buyer(),account_object.get_seller(),None)
				};
			}
			todo!()
		}
	}


}
