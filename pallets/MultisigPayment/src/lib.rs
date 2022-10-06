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

//#[cfg(test)]
//mod mock;

//#[cfg(test)]
//mod tests;
mod helper;


//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

// A multi-signature pallet implemented for `Vane Payment System`

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::{BoundedSlice, traits::{StaticLookup}};
	use super::helper::{AccountSigners, Order, RevertReasons, Confirm};

	pub(super) type AccountFor<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

	}

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_allowed_signers)]
	pub(super) type AllowedSigners<T: Config> = StorageValue<_,AccountSigners<T>>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_signers)]
	pub(super) type Signers<T: Config> = StorageValue<_,Vec<T::AccountId>,ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>{
		CallExecuted{
			multi_id: T::AccountId,
			timestamp: T::BlockNumber,

		},
		AccountAddressStored(T::AccountId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// A call that transfers fund from a buyer to a multi-owned account.
		//

		// Tasks:
		// 1. Add filter to the call so that only an origin from AccountSigner can be an origin
		#[pallet::weight(10)]
		pub fn vane_pay(
			origin: OriginFor<T>,
			//reference: Option<Order<T>>,
			payee: Option<AccountFor<T>>,

		) -> DispatchResult {

			let buyer = ensure_signed(origin)?;
			Ok(())

		}

		// If the payer accidently makes a mistake due to RevertReasons the funds can be refunded back
		// Punishment will occur if the reason is personal.
		#[pallet::weight(10)]
		pub fn revert_fund(origin:OriginFor<T>, reason:RevertReasons) -> DispatchResult{
			Ok(())
		}

		// Get the confirm account address and store them in Signers Storage Item. Sort and make sure
		// buyer's address is first
		// Always make sure if its the buyer, he should be first in the vector,
		// 		1. Store the account_id in the Signer Storage Item,
		// 		2. Then next steps will follow after this,

		#[pallet::weight(10)]
		pub fn confirm(origin: OriginFor<T>, who: Confirm) ->DispatchResult{
			let who = ensure_signed(origin)?;

			let bounded_keys = Vec::<AccountFor<T>>::try_from((who).clone());
			
			// Storing confirmed account address
			Signers::<T>::put(bounded_keys);

			// Emitting storage event.
			Self::deposit_event(Event::AccountAddressStored(who));
			// Return a successful DispatchResultWithPostInfo
			
			Ok(())
		}
	}

}
