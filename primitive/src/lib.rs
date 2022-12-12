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

