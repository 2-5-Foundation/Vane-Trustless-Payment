#![cfg_attr(not(feature = "std"), no_std)]


mod helper;

pub use pallet::*;

#[frame_support::pallet]
mod pallet{
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use frame_support::traits::Randomness;
	use crate::helper::utils::{Confirm, VaneAccountData};

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_randomness_collective_flip::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(core::marker::PhantomData<T>);

	#[pallet::error]
	pub enum Error<T>{
		AccountAlreadyRegistered,

	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>{
		UserRegistered{
			id: T::AccountId,
			time: T::BlockNumber
		}
	}

	#[pallet::storage]
	pub type TotalUsers<T: Config> = StorageValue<_,u32,ValueQuery>;

	#[pallet::storage]
	pub type PayeeStorage<T:Config> =
	         StorageMap<_,Blake2_128,T::AccountId,VaneAccountData<T>>;

	#[pallet::storage]
	pub type PayerStorage<T: Config> =
			 StorageMap<_,Blake2_128,T::AccountId,VaneAccountData<T>>;


	#[pallet::call]
	impl<T: Config> Pallet<T>{
		#[pallet::weight(10)]
		pub fn register(origin: OriginFor<T>,who: Confirm) -> DispatchResult{
			let signer = ensure_signed(origin)?;
			ensure!(PayerStorage::<T>::contains_key(signer.clone()),Error::<T>::AccountAlreadyRegistered);

			match who {
				Confirm::Payee => {
					// Check if its already registered

					let ref_no = <TotalUsers<T>>::get() ;
					let user_data = VaneAccountData{
						reference: Some(ref_no),
						account: signer.clone(),
					};
					PayeeStorage::<T>::insert(&signer,user_data);
					let time = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::<T>::UserRegistered {id:signer,time})
				},
				Confirm::Payer => {
					let user_data = VaneAccountData{
						reference: None,
						account: signer.clone(),
					};
					PayerStorage::<T>::insert(&signer,user_data);
					let time = <frame_system::Pallet<T>>::block_number();
					Self::deposit_event(Event::<T>::UserRegistered {id:signer,time})
				}
			};
			<TotalUsers<T>>::put(<TotalUsers<T>>::get()+1);
			Ok(())
		}
	}

}
