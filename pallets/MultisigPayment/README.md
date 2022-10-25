License: Unlicense

# Pallet Vane-Payment
A multi-signature payment and verification implementation.

### Description
This is the first iteration of pallet vane-payment. It includes `fn vane_pay` and `confim_pay` extrinsics.

> lib.rs

##### `vane_pay`
**Status** : First beta release

This is the initial function to be called when doing payments. It will create a multi-sig account from caller's id and payee's id along with resolving mechanism chosen.

Account signers struct object will be created and stored inside allowed signers storage item.

The funds will be transfered from caller's account to the multi-sig account created.

##### `confirm_pay`
**Status** : First beta release

This is the second function to be called. The order of calling should start from payee to payer. This is because of the arrangment of id's inside allowed signers.

This function stores the caller id inside Confirmed_Signers storage item. The id's will be used to derive a multi-id and match the created multi-id against the allowed multi-id created earlier.

If the checks confirms transfer inner function will be dispatched to send all funds from multi-sig created account to the payee's account.

###### Extras

This ensures that the payee's account which the payer intended to send is valid. And this feature it will be crucial for sub-harbour trustless and permissionless e-commerce protocol which will introduce a new way people exchange goods and value online. Bringing trustless payments.

#### `revert_fund`
**Status** : Under development
This will introduce a way to revert wrongfully sent transaction to a wrong payee's address.

Also it will add a way to punish payers and payees if too many mistakes are conducted in a specific duration of blocktimes.



---


#### Down in the line upcoming features
1. Introducing legal_team account resolving mechanism
2. Introducing governance mechanism for resolving disputes ( This feature is useful in trade payments ).
3. Advancing vane-pay and confirm-pay functions to work with intended sub-harbour e-commerce protocol to achieve trustless payments.



