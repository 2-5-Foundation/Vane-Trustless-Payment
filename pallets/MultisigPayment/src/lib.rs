#![cfg_attr(not(feature = "std"), no_std)]
//------------Inner descriptions-----------------------------------------//
// The pallet should be generic
// The main extrinsic is the multisig Call which consist of following inputs;
//          -origin (signed)
//          -reference = Option<>
//          -payee_address = Option<>
//
// The 'reference' should have an account_id associated with it.
// The call mainly intention is to be used when paying for a product
// whereby a seller and a buyer are the participants of the multi-sig call.
// But it can be used in any other usecases provided that
// the usecase marries the call requirements.
//
// What does the call do? inner function set_callers()
// will register the account_ids needed for making
// the call. First id will be the origin, second will be from reference object.
//
// The inner Call is balance's call transfer function.


pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

//! A multi-signature pallet implemented for `Vane Payment System`


#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{StaticLookup}
	};

	pub(super) type AccountFor<T> = <T::Lookup as StaticLookup>::Source;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::storage]
	#[pallet::getter(fn get_signers)]
	pub(super) type Signers<T: Config> = StorageValue<_,AccountSigners<T>>;

	#[derive(Encode, Decode, Clone, Copy, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub struct AccountSigners<T>{
		buyer: AccountFor<T>,
		seller: AccountFor<T>,
		resolver: Resolver<T>,
	}

	#[derive(Encode, Decode, Clone, Copy, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum Resolver<T>{
		legal_team(AccountFor<T>),
		governance,
		//some future time feature
		both(AccountFor<T>)
	}



	#[derive(Encode, Decode, Clone, Copy, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub struct Order<T>{
		order_number:u32,
		account: AccountFor<T>
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10)]
		pub fn vane_pay(
			origin: OriginFor<T>,
			reference: Option<Order<T>>,
			payee: T::AccountId
		) -> DispatchResult {

			let buyer = ensure_signed(origin)?;
			Ok(())

		}
	}

	//--- Helper functions---------------------------------

	impl<T:Config> Pallet<T>{
		pub fn set_signers() ->DispatchResult{
			todo!()
		}
	}
}
