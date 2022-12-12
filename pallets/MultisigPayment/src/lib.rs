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

mod helper;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod benchmarking;

//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

// A multi-signature implementation for `Vane Payment System`

#[frame_support::pallet]
pub mod pallet {
	use super::helper::{AccountSigners, CallExecuted, Confirm, ResolverChoice, RevertReasons};
	use frame_support::{pallet, pallet_prelude::*, traits::tokens::currency::Currency};
	use frame_system::pallet_prelude::*;
	use vane_primitive::OrderTrait;
	use sp_io::hashing::blake2_256;
	use sp_runtime::{
		parameter_types,
		traits::{StaticLookup, TrailingZeroInput},
	};
	use sp_std::vec::Vec;


	pub(super) type AccountOf<T> = <T as frame_system::Config>::AccountId;
	pub(super) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountOf<T>>>::Balance;

	// Max signers for Confirm Signers Bounded Vec
	parameter_types! {
		pub const MaxSigners: u16 = 2;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config  {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//type Order: OrderTrait + TypeInfo + Decode + Encode + Clone + PartialEq + Debug;
		type Currency: Currency<Self::AccountId>;
	}

	// Not yet implemented
	#[pallet::storage]
	#[pallet::getter(fn get_resolver)]
	pub(super) type ResolverSigner<T: Config> = StorageValue<_, T::AccountId>;

	// Number of multi-sig transactions done by a specific account_id
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_account_multitxns)]
	pub(super) type AccountMultiTxns<T: Config> =
		StorageMap<_, Blake2_256, T::AccountId, Vec<CallExecuted<T>>, ValueQuery>;

	// Introduced StorageMap because this storage should contain more  than one instance of
	// AccountSigners

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_allowed_signers)]
	pub(super) type AllowedSigners<T: Config> =
		StorageMap<_, Blake2_256, T::AccountId, AccountSigners<T>>;

	// 1. Change confirmed signers to be StorageMap and key to be the payer's address
	// 2. Change the way confirmed addresses are stored and make sure the payer starts
	// 3. and clear the storage once the call is executed, clear multi_id account and confirmed and
	// allowed signers
	#[pallet::storage]
	#[pallet::getter(fn get_signers)]

	// Jibril change it to be storageMap and key to be multi_id,
	// Introduce new stoarge value for multi_id created from Allowed Signers
	pub(super) type ConfirmedSigners<T: Config> =
		StorageMap<_,Twox64Concat, u32, BoundedVec<T::AccountId, MaxSigners>, ValueQuery>;

	// Number of reverted or faulty transaction a payer did
	#[pallet::storage]
	#[pallet::getter(fn get_failed_txn_payer)]
	pub(super) type RevertedTxnPayer<T: Config> =
		StorageMap<_, Blake2_256, T::AccountId, u32, ValueQuery>;

	// Number of reverted or faulty transaction a payee did
	#[pallet::storage]
	#[pallet::getter(fn get_failed_txn_payee)]
	pub(super) type RevertedTxnPayee<T: Config> =
		StorageMap<_, Blake2_256, T::AccountId, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CallExecuted {
			multi_id: T::AccountId,
			timestamp: T::BlockNumber,
		},

		MultiAccountCreated {
			account_id: T::AccountId,
			timestamp: T::BlockNumber,
		},

		BalanceTransferredAndLocked {
			to_multi_id: T::AccountId,
			from: T::AccountId,
			timestamp: T::BlockNumber,
		},

		PayeeAddressConfirmed {
			account_id: T::AccountId,
			timestamp: T::BlockNumber,
		},

		PayerAddressConfirmed {
			account_id: T::AccountId,
			timestamp: T::BlockNumber,
		},

		SubmittedPayment {
			from_account: T::AccountId,
			to_account: T::AccountId,
			amount: BalanceOf<T>,
			resolver: Option<ResolverChoice>,
			timestamp: T::BlockNumber,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		// Any system error
		UnexpectedError,

		FailedToMatchAccounts,

		MultiAccountExists,

		ExceededSigners,

		AccountAlreadyExist,

		WaitForPayeeToConfirm,

		WaitForPayerToConfirm,

		PayerAlreadyConfirmed,

		PayeeAlreadyConfirmed,

		NotAllowedPayeeOrPaymentNotInitialized,

		MultiSigCallFailed,
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
			payee: Option<T::AccountId>,
			amount: BalanceOf<T>,
			resolver: ResolverChoice,
			// Third parameter will be a type that implements Order trait from primitive
			//order: Option<T::Order>
		) -> DispatchResult {
			// 1. Check if the Payee is in the Register Storage
			// 2.
			let payer = ensure_signed(origin)?;
			let payee = payee.ok_or(Error::<T>::UnexpectedError)?;


			match resolver {
				ResolverChoice::None => {
					Self::inner_vane_pay_wo_resolver(payer.clone(), payee.clone(), amount)?;
					let time = <frame_system::Pallet<T>>::block_number();

					Self::deposit_event(Event::SubmittedPayment {
						from_account: payer,
						to_account: payee,
						amount,
						resolver: None,
						timestamp: time,
					})
				},
				_ => (),
			}

			Ok(())
		}

		// Get the confirm account address and store them in Signers Storage Item. Sort and make
		// sure buyer's address is first
		// Always make sure if its the buyer, he should be first in the vector,
		// 		1. Store the account_id in the Signer Storage Item,
		// 		2. Then next steps will follow after this,

		#[pallet::weight(10)]
		pub fn confirm_pay(origin: OriginFor<T>, who: Confirm, reference_no: u32) -> DispatchResult {
			// 1. Check if 0 index is a occupied and if true check if its a Payee if true return Err
			// 2. If its not a Payee then add new account which it will be a Payer
			// 3. If index 0 is not occupied then check if the address is a Payer, if its true
			// return Err 4. If the address is a Payee then push it to the vec.
			//---------------------------------------------------------------------------------------//
			// This will ensure that in 0th index is always Payee address and the Payer cannot
			// confirm first

			let user_account = ensure_signed(origin)?;
			// Check the storage
			let b_vec = ConfirmedSigners::<T>::get(reference_no);


			if let Some(addr) = b_vec.get(0) {
				if addr.eq(&user_account) {
					return Err(Error::<T>::PayeeAlreadyConfirmed.into())

					// Else for checking if payee tries to confirm twice.
				} else {

					ConfirmedSigners::<T>
						::try_mutate(reference_no, |vec| vec.try_push(user_account.clone()))
						.map_err(|_| Error::<T>::ExceededSigners)?;

					let time = <frame_system::Pallet<T>>::block_number();

					Self::deposit_event(Event::PayerAddressConfirmed {
						account_id: user_account,
						timestamp: time,
					});

					// Construct AccountSigner object from ConfirmedSigners storage

					let confirmed_acc_signers = AccountSigners::<T>::new(
						ConfirmedSigners::<T>::get(reference_no)
							.get(0)
							.ok_or(Error::<T>::UnexpectedError)?
							.clone(),

						ConfirmedSigners::<T>::get(reference_no)
							.get(1)
							.ok_or(Error::<T>::UnexpectedError)?
							.clone(),
						// The default resolver is none but logic will be made to be customizable
						None,
					);

					// Derive the multi_id of newly constructed AccountSigner and one from
					// AllowedSigners
					let confirmed_multi_id = Self::derive_multi_id(confirmed_acc_signers);

					// Get the AllowedSigners from storage
					let payer = ConfirmedSigners::<T>::get(reference_no)
						.get(1)
						.ok_or(Error::<T>::UnexpectedError)?
						.clone();

					let payee = ConfirmedSigners::<T>::get(reference_no)
						.get(0)
						.ok_or(Error::<T>::UnexpectedError)?
						.clone();

					let allowed_signers = AllowedSigners::<T>::get(payer.clone())
						.ok_or(Error::<T>::NotAllowedPayeeOrPaymentNotInitialized)?;

					let allowed_multi_id = Self::derive_multi_id(allowed_signers);
					// Compute the hash of both multi_ids (proof)
					if confirmed_multi_id.eq(&allowed_multi_id) {
						let encoded_proof = (allowed_multi_id.clone(), confirmed_multi_id.clone())
							.using_encoded(blake2_256);
						let proof =
							Decode::decode(&mut TrailingZeroInput::new(encoded_proof.as_ref()))
								.map_err(|_| Error::<T>::UnexpectedError)?;

						Self::dispatch_transfer_call(
							proof,
							payer,
							payee,
							allowed_multi_id,
							confirmed_multi_id,
						)?;
					} else {
						return Err(Error::<T>::FailedToMatchAccounts.into())
					}
				}

			// Else block from If let Some()
			} else {
				match who {
					Confirm::Payer => return Err(Error::<T>::WaitForPayeeToConfirm.into()),

					Confirm::Payee => {
						ConfirmedSigners::<T>::try_mutate(reference_no,|vec| vec.try_push(user_account.clone()))
							.map_err(|_| Error::<T>::ExceededSigners)?;

						let time = <frame_system::Pallet<T>>::block_number();

						Self::deposit_event(Event::PayeeAddressConfirmed {
							account_id: user_account,
							timestamp: time,
						});
					},
				};
			};

			Ok(())
		}

		// If the payer accidently makes a mistake due to RevertReasons the funds can be refunded
		// back Punishment will occur if the reason is personal.

		// We should introduce some sort of limit for WrongAddress reason occurrence.
		#[pallet::weight(10)]
		pub fn revert_fund(origin: OriginFor<T>, reason: RevertReasons) -> DispatchResult {
			Ok(())
		}
	}
}
