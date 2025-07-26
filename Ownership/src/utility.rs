use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::bytes::Bytes;
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_sdk::prelude::*;

sol! {


    error ONLY_OWNER(address owner);
    error ALREADY_REGISTERED(address caller);
    error ADDRESS_ZERO(address zero);
    error CODE_ALREADY_GENERATED();
    error UNAUTHORIZED(address caller);
    error ITEM_DOESNT_EXIST(string);
    error DOES_NOT_EXIST();
    error CONTRACT_DOEST_NOT_EXIST();
    error NAME_ALREADY_EXIST(string);
    error INVALID_SIGNATURE();
    error ITEM_CLAIMED_ALREADY(string itemId);
    error ITEM_NOT_CLAIMED_YET();
    error NOT_REGISTERED(address user);
    error NAME_NOT_AVAILABLE(string username);
    error USER_DOES_NOT_EXIST(address user);
    error CANNOT_GENERATE_CODE_FOR_YOURSELF(address caller);
    error USERNAME_MUST_BE_AT_LEAST_3_LETTERS();
    error INVALID_MANUFACTURER_NAME(string);
    error AUTHENTICITY_NOT_SET();

    event ContractCreated(address indexed contractAddress,address indexed owner);
    event UserRegistered(address indexed userAddress, string indexed username);
    event OwnershipCode(bytes32 indexed ownershipCode,address indexed tempOwner);
    event ItemCreated(string indexed itemId, address indexed owner);
    event OwnershipClaimed(address indexed newOwner, address indexed oldOwner);
    event CodeRevoked(bytes32 indexed itemHash);
    event AuthenticitySet(address indexed authenticityAddress);
}

#[derive(Debug)]
pub struct Certificate {
    pub name: String,
    pub unique_id: String,
    pub serial: String,
    pub date: U256,
    pub owner: Address,
    pub metadata_hash: Bytes,
    pub metadata: Vec<String>,
}

#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub item_id: String,
    pub serial: String,
    pub date: U256,
    pub owner: Address,
    pub manufacturer: String,
    pub metadata: Vec<String>,
}

#[derive(SolidityError)]
pub enum EriError {
    OnlyOwner(ONLY_OWNER),
    AddressZero(ADDRESS_ZERO),
    Registered(ALREADY_REGISTERED),
    BadUsername(USERNAME_MUST_BE_AT_LEAST_3_LETTERS),
    NotAvailable(NAME_NOT_AVAILABLE),
    NotExist(USER_DOES_NOT_EXIST),
    AuthenticityNotSet(AUTHENTICITY_NOT_SET),
    Unauthorized(UNAUTHORIZED),
    NotRegistered(NOT_REGISTERED),
    AlreadyClaimed(ITEM_CLAIMED_ALREADY),
    CannotGenerate(CANNOT_GENERATE_CODE_FOR_YOURSELF),
    NotClaimed(ITEM_NOT_CLAIMED_YET),
    DoesNotExist(DOES_NOT_EXIST)
    
}
