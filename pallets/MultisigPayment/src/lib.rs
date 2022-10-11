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

//------------------------------------------------------------------------------------------//
// We must keep track of seller and buyer bad behaviours in storage item so that we can introduce
// further punishments when bad repeated behaviour occurs

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod helper;


//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

// A multi-signature implementation for `Vane Payment System`

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::{parameter_types, traits::{StaticLookup}};
	use primitive::OrderTrait;
	use super::helper::{
		AccountSigners,
		RevertReasons,
		Confirm,
		ResolverChoice,
	};

	pub(super) type AccountFor<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	parameter_types! {
		pub const MaxSigners: u16 = 2;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//type Order: OrderTrait + TypeInfo + Decode + Encode + Clone + PartialEq + Debug;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_resolver)]
	pub(super) type ResolverSigner<T: Config> = StorageValue<_,T::AccountId>;


	// Introduced StorageMap because this storage should contain more  than one instance of AccountSigners

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_allowed_signers)]
	pub(super) type AllowedSigners<T: Config> = StorageMap<_,Blake2_256,T::AccountId,AccountSigners<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_signers)]
	pub(super) type ConfirmedSigners<T: Config> = StorageValue<_,BoundedVec<T::AccountId, MaxSigners>,ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>{
		CallExecuted{
			multi_id: T::AccountId,
			timestamp: T::BlockNumber,

		},
		PayeeAddressConfirmed{
			account_id:T::AccountId,
			timestamp: T::BlockNumber,
		},
		PayersAddressConfirmed{
			account_id: T::AccountId,
			timestamp: T::BlockNumber,
		}

	}
	#[pallet::error]
	pub enum Error<T>{
		UnexpectedError,
		MultiAccountExists,
		ExceededSigners,
		AccountAlreadyExist,
		AccountNotFound,
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
			payee: Option<AccountFor<T>>,
			resolver: ResolverChoice,
			// Third parameter will be a type that implements Order trait from primitive
			//order: Option<T::Order>
		) -> DispatchResult {

			let payer = ensure_signed(origin)?;
			let payee = payee.ok_or(Error::<T>::UnexpectedError)?;
			let payee = <<T as frame_system::Config>::Lookup as StaticLookup>
														::lookup(payee)?;

			match resolver {
				ResolverChoice::LegalTeam => Self::inner_vane_pay_wo_resolver(payer,payee)?,
				ResolverChoice::Governance=> ()
			}
			Ok(())
		}

		// If the payer accidently makes a mistake due to RevertReasons the funds can be refunded back
		// Punishment will occur if the reason is personal.

		// We should introduce some sort of limit for WrongAddress reason occurrence.
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
		pub fn confirm_pay(origin: OriginFor<T>, who: Confirm) ->DispatchResult{

			let user_account = ensure_signed(origin)?;

			// Check if account already exists

			// Check the account from Payer and Payee are the one registered in AllowedSigners storage
			// This is crucial and this check should occur before the match statement
				match who {
					Confirm::Payee => {

						<ConfirmedSigners<T>>::try_mutate(|account_vec| {

							account_vec.try_insert(0,user_account.clone())

						}).map_err(|_| Error::<T>::ExceededSigners)?;

						let timestamp = <frame_system::Pallet<T>>::block_number();
						Self::deposit_event(Event::PayeeAddressConfirmed {account_id:user_account, timestamp});

					},
					Confirm::Payer => {
						<ConfirmedSigners<T>>::try_mutate(|account_vec| {

							account_vec.try_insert(1,user_account.clone())

						}).map_err(|_| Error::<T>::ExceededSigners)?;

						let timestamp = <frame_system::Pallet<T>>::block_number();
						Self::deposit_event(Event::PayersAddressConfirmed{account_id:user_account, timestamp});

					},
				};

			// We will discuss first my head is spinning,I want to get the logic right
			// Next steps are
			//     1. After registering the specific account to the storage,
			//        construct the AccountSigners object using those accounts
			//     2. Derive the multi_id Account from those accounts using the Account Signers
			//       object constructed
			//     3. Hash the multi_id account
			//     4. Repeat the same process for


			let mut confirmed_signers = <ConfirmedSigners<T>>::get();
			let pos = confirmed_signers
				.binary_search(&user_account)
				.ok()
				.ok_or(Error::<T>::AccountNotFound)?;
			let account_signer = AccountSigners::<T>
								::new(payer:confirmed_signers[pos].0, 
									payee:confirmed_signers[pos].1,
									resolver:Resolver.LegalTeam}
			
			let multi_id = derive_multi_id(account_signer.clone());
			

			Ok(())
		}
	}

}
