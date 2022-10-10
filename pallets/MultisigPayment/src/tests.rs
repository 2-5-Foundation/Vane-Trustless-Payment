use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use crate::helper::{AccountSigners, Confirm};


// A testing Account Object
pub fn new_acc(payer:u64,payee:u64) -> AccountSigners<Test>{
	let acc = AccountSigners::<Test>::new(
		payer,
		payee,
		None
	);
	acc
}


// Testing Deriving Multi_Id Account
#[test]
fn derive_multi_id_test(){
	new_test_ext().execute_with(||{
	    let acc = new_acc(1,2);
		assert_eq!(VanePayment::derive_multi_id(acc),11691055940168842723);

	})
}


// Testing Account formation and storage
#[test]
fn multi_acc_formation_storage_test(){
	new_test_ext().execute_with(||{
		let acc = new_acc(1,2);
		let multi_id = VanePayment::derive_multi_id(acc);
		assert_ok!(VanePayment::create_multi_account(multi_id));

		// Checking the account storage in frame_system;
		assert!(System::account_exists(&multi_id));
		assert_eq!(System::consumers(&multi_id),1);

		// This should fail because the account is already registered;
		assert_noop!(VanePayment::create_multi_account(multi_id),Error::<Test>::MultiAccountExists);
	})
}


// Testing Account Confirmation (Payee and Payer) and storage.
#[test]
fn acc_confirmation(){
	new_test_ext().execute_with(||{

		// Configuring account storage;

		let acc = new_acc(1,2);
		let multi_id = VanePayment::derive_multi_id(acc);
		assert_ok!(VanePayment::create_multi_account(multi_id));

		// Payer and Payee confirmation;
		// Payee confirmation
		assert_ok!(VanePayment::confirm_pay(Origin::signed(2),Confirm::Payee));
		// Payer confirmation
		assert_ok!(VanePayment::confirm_pay(Origin::signed(1),Confirm::Payer));

		// Checking storage
		assert_eq!(VanePayment::get_signers(),vec![1,2]);

		// This should fail
		assert_noop!(VanePayment::confirm_pay(Origin::signed(3),Confirm::Payer),
			Error::<Test>::ExceededSigners);

		assert_noop!(VanePayment::confirm_pay(Origin::signed(2),Confirm::Payer),
			Error::<Test>::ExceededSigners);

		assert_noop!(VanePayment::confirm_pay(Origin::signed(1),Confirm::Payer),
			Error::<Test>::ExceededSigners);




	})
}


// Checking multi-sig Call for an individual payee only.




// Checking multi-sig call for a seller.



// Checking Handling reverting for a payer.



// Handling reverting for a Multi-SIg call involving a seller




// Checking dispute handling by a legal team



// Checking handling dispute by governance

