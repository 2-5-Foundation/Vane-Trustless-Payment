#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::*;
use frame_support::pallet_prelude::*;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use codec::MaxEncodedLen;

pub trait OrderTrait {
	fn get_seller() {}
	fn get_order_number() {}
	fn get_delivery_time() {}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct VaneAccountData<Balance,Reference>{
	/// Non-reserved part of the balance. There may still be restrictions on this, but it is the
	/// total pool what may in principle be transferred, reserved and used for tipping.
	///
	/// This is the only balance that matters in terms of most operations on tokens. It
	/// alone is used to determine the balance when in the contract execution environment.
	pub free: Balance,
	/// Balance which is reserved and may not be used at all.
	///
	/// This can still get slashed, but gets slashed last of all.
	///
	/// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
	/// that are still 'owned' by the account holder, but which are suspendable.
	/// This includes named reserve and unnamed reserve.
	pub reserved: Balance,
	/// The amount that `free` may not drop below when withdrawing for *anything except transaction
	/// fee payment*.
	pub misc_frozen: Balance,
	/// The amount that `free` may not drop below when withdrawing specifically for transaction
	/// fee payment.
	pub fee_frozen: Balance,
	/// The reference number which indicate certain transaction inside multi-sig call
	pub reference: Reference,
}
