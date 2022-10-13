use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use crate::helper::{AccountSigners, Confirm};


// A testing Account Object
pub fn new_acc(payee:u64,payer:u64) -> AccountSigners<Test>{
	let acc = AccountSigners::<Test>::new(
		payee,
		payer,
		None
	);
	acc
}


// Testing Deriving Multi_Id Account
#[test]
fn derive_multi_id_test(){
	new_test_ext().execute_with(||{
	    let acc = new_acc(2,1);
		assert_eq!(VanePayment::derive_multi_id(acc),3149924236044933178);

	})
}


// Testing Account formation and storage
#[test]
fn multi_acc_formation_storage_test(){
	new_test_ext().execute_with(||{
		let acc = new_acc(2,1);
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

		let acc = new_acc(2,1);
		let multi_id = VanePayment::derive_multi_id(acc);
		assert_ok!(VanePayment::create_multi_account(multi_id));

		// Payer and Payee confirmation;
		// Payer confirmation first should fail
		assert_noop!(VanePayment::confirm_pay(Origin::signed(1),Confirm::Payer),Error::<Test>::WaitForPayeeToConfirm);
		// Payee confirmation should work
		assert_ok!(VanePayment::confirm_pay(Origin::signed(2),Confirm::Payee));
		// Payee re-confirmation should fail
		assert_noop!(VanePayment::confirm_pay(Origin::signed(2),Confirm::Payee), Error::<Test>::PayeeAlreadyConfirmed);
		// Payer Confirmation
		assert_ok!(VanePayment::confirm_pay(Origin::signed(1), Confirm::Payer));


		// Checking storage
		assert_eq!(VanePayment::get_signers(),vec![2,1]);

		// This should fail
		assert_noop!(VanePayment::confirm_pay(Origin::signed(3),Confirm::Payer),
			Error::<Test>::ExceededSigners);




	})
}


// Testing inner_vane_pay_wo_resolver
#[test]
fn inner_vane_pay_wo_resolver_test(){
	new_test_ext().execute_with(||{
		// Multi Account Id
		let acc = new_acc(5,1);
		let multi_id = VanePayment::derive_multi_id(acc);
		assert_ok!(VanePayment::inner_vane_pay_wo_resolver(1,5,100000));

		// Checking Multi_Id balance
		assert_eq!(Balances::free_balance(multi_id),100000);
		assert_eq!(Balances::free_balance(5),1000);


	})
}

// Checking multi-sig Call for an individual payee only.




// Checking multi-sig call for a seller.



// Checking Handling reverting for a payer.



// Handling reverting for a Multi-SIg call involving a seller




// Checking dispute handling by a legal team



// Checking handling dispute by governance

