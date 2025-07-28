// Gotten from: https://stylus-by-example.org/basic_examples/verify_signature

extern crate alloc;

use crate::utility::{EriError::*, *};
use alloc::string::String;
use alloy_primitives::FixedBytes;
use alloy_sol_types::{
    sol_data::{Address as SOLAddress, FixedBytes as SolFixedBytes, *},
    SolType,
};

use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{address, Address, U256},
    call::{self, Call},
    crypto::keccak,
    prelude::*,
};
use stylus_sdk::call::static_call;
// impl VerifySignature {
/* 1. Unlock MetaMask account
ethereum.enable()
*/

/* 2. Get message hash to sign
getMessageHash(
    0x14723A09ACff6D2A60DcdF7aA4AFf308FDDC160C,
    123,
    "coffee and donuts",
    1
)

hash = "0xcf36ac4f97dc10d91fc2cbb20d718e94a8cbfe0f82eaedc6a4aa38946fb797cd"
*/

fn get_message_hash(
    name: String,
    unique_id: String,
    serial: String,
    date: U256,
    owner: Address,
    metadata_hash: FixedBytes<32>,
) -> FixedBytes<32> {
    let message_data = [
        name.as_bytes(),
        unique_id.as_bytes(),
        serial.as_bytes(),
        &date.to_be_bytes_vec(),
        &owner.to_vec(),
        &metadata_hash.to_vec(),
    ]
    .concat();
    keccak(message_data).into()
}

/* 3. Sign message hash
# using browser
account = "copy paste account of signer here"
ethereum.request({ method: "personal_sign", params: [account, hash]}).then(console.log)

# using web3
web3.personal.sign(hash, web3.eth.defaultAccount, console.log)

Signature will be different for different accounts
0x993dab3dd91f5c6dc28e17439be475478f5635c92a56e17e82349d3fb2f166196f466c0b4e0c146f285204f0dcb13e5ae67bc33f4b888ec32dfe0a063e8f3f781b
*/
fn get_eth_signed_message_hash(message_hash: FixedBytes<32>) -> FixedBytes<32> {
    const SIGNED_MESSAGE_HEAD: &'static str = "\x19Ethereum Signed Message:\n32";

    let message_to_be_decoded = [SIGNED_MESSAGE_HEAD.as_bytes(), &message_hash.to_vec()].concat();
    keccak(message_to_be_decoded).into()
}

/* 4. Verify signature
signer = 0xB273216C05A8c0D4F0a4Dd0d7Bae1D2EfFE636dd
to = 0x14723A09ACff6D2A60DcdF7aA4AFf308FDDC160C
amount = 123
message = "coffee and donuts"
nonce = 1
signature =
    0x993dab3dd91f5c6dc28e17439be475478f5635c92a56e17e82349d3fb2f166196f466c0b4e0c146f285204f0dcb13e5ae67bc33f4b888ec32dfe0a063e8f3f781b
*/
pub fn verify(
    // expected_signer: Address,
    name: String,
    unique_id: String,
    serial: String,
    date: U256,
    owner: Address,
    metadata_hash: FixedBytes<32>,
    signature: Bytes,
) -> Result<bool, EriError> {
    let message_hash = get_message_hash(name, unique_id, serial, date, owner, metadata_hash);

    let eth_signed_message_hash = get_eth_signed_message_hash(message_hash);

    match recover_signer(eth_signed_message_hash, signature) {
        Ok(recovered_signer) => Ok(recovered_signer == owner),
        Err(err) => Err(InvalidSignature(INVALID_SIGNATURE {})),
    }
}

fn recover_signer(
    eth_signed_message_hash: FixedBytes<32>,
    signature: Bytes,
) -> Result<Address, EriError> {
    let (r, s, v) = split_signature(signature);
    ec_recover_call(eth_signed_message_hash, v, r, s)
}

/// Invoke the ECRECOVER precompile.
fn ec_recover_call(
    hash: FixedBytes<32>,
    v: u8,
    r: FixedBytes<32>,
    s: FixedBytes<32>,
) -> Result<Address, EriError> {
    type ECRECOVERType = (
        SolFixedBytes<32>,
        Uint<8>,
        SolFixedBytes<32>,
        SolFixedBytes<32>,
    );

    const EC_RECOVER: Address = address!("0000000000000000000000000000000000000001");

    let data = (hash, v, r, s);
    let encoded_data = ECRECOVERType::abi_encode(&data);

    match call::static_call(Call::new(), EC_RECOVER, &encoded_data) {
        Ok(result) => Ok(SOLAddress::abi_decode(&result, false).unwrap()),
        Err(_) => Err(ECRecoverError(EC_RECOVER_CALL_ERROR {})),
    }
}

fn split_signature(signature: Bytes) -> (FixedBytes<32>, FixedBytes<32>, u8) {
    let r = FixedBytes::from_slice(&signature[0..32]);
    let s = FixedBytes::from_slice(&signature[32..64]);
    let v = signature[64];
    (r, s, v)
}
