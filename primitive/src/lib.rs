#![cfg_attr(not(feature = "std"), no_std)]

pub trait OrderTrait{
	fn get_seller(){}
	fn get_order_number(){}
	fn get_delivery_time(){}
}
