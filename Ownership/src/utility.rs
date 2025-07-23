use alloy_sol_types::sol;
use stylus_sdk::prelude::*;

sol! {
    error ONLY_OWNER(address owner);
    error ALREADY_REGISTERED(address caller);
    error ADDRESS_ZERO(address zero);
    error CODE_ALREADY_GENERATED();
    error UNAUTHORIZED(address);
    error ITEM_DOESNT_EXIST(string);
    error DOES_NOT_EXIST();
    error CONTRACT_DOEST_NOT_EXIST();
    error NAME_ALREADY_EXIST(string);
    error INVALID_SIGNATURE();
    error ITEM_CLAIMED_ALREADY(string);
    error ITEM_NOT_CLAIMED_YET();
    error NOT_REGISTERED(address);
    error NAME_NOT_AVAILABLE(string username);
    error USER_DOES_NOT_EXIST(address);
    error CANNOT_GENERATE_CODE_FOR_YOURSELF(address);
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

#[derive(SolidityError)]
pub enum EriError {
    OnlyOwner(ONLY_OWNER),
    AddressZero(ADDRESS_ZERO),
    Registered(ALREADY_REGISTERED),
    BadUsername(USERNAME_MUST_BE_AT_LEAST_3_LETTERS),
    NotAvailable(NAME_NOT_AVAILABLE)
}