use alloy_sol_types::sol;
use stylus_sdk::prelude::SolidityError;

sol! {

    struct EIP712Domain {
        string name;
        string version;
        uint256 chainId;
        address verifyingContract;
    }

    error ONLY_OWNER(address owner);
    error ALREADY_REGISTERED(address user);
    error ADDRESS_ZERO(address zero);
    error CODE_ALREADY_GENERATED();
    error UNAUTHORIZED(address user);
    error ITEM_DOESNT_EXIST(string itemId);
    error DOES_NOT_EXIST();
    error CONTRACT_DOEST_NOT_EXIST();
    error NAME_ALREADY_EXIST(string name);
    error INVALID_SIGNATURE();
    error ITEM_CLAIMED_ALREADY(string itemId);
    error ITEM_NOT_CLAIMED_YET();
    error NOT_REGISTERED(address user);
    error NAME_NOT_AVAILABLE(string name);
    error USER_DOES_NOT_EXIST(address user);
    error CANNOT_GENERATE_CODE_FOR_YOURSELF(address user);
    error USERNAME_MUST_BE_AT_LEAST_3_LETTERS();
    error INVALID_MANUFACTURER_NAME(string name);
    error AUTHENTICITY_NOT_SET();


    event ManufacturerRegistered(address indexed manufacturerAddress, string indexed manufacturerName);
    event ContractCreated(address indexed contractAddress, address indexed owner);
}

#[derive(SolidityError)]
pub enum EriError {
    OnlyOwner(ONLY_OWNER),
    AddressZero(ADDRESS_ZERO),
    Registered(ALREADY_REGISTERED),
    BadUsername(USERNAME_MUST_BE_AT_LEAST_3_LETTERS),
    NotExist(USER_DOES_NOT_EXIST),
    AuthenticityNotSet(AUTHENTICITY_NOT_SET),
    Unauthorized(UNAUTHORIZED),
    NotRegistered(NOT_REGISTERED),
    AlreadyClaimed(ITEM_CLAIMED_ALREADY),
    CannotGenerate(CANNOT_GENERATE_CODE_FOR_YOURSELF),
    NotClaimed(ITEM_NOT_CLAIMED_YET),
    DoesNotExist(DOES_NOT_EXIST),
    ItemDoesNotExist(ITEM_DOESNT_EXIST),
    InvalidManufacturerName(INVALID_MANUFACTURER_NAME),
    NameNotAvailable(NAME_NOT_AVAILABLE)
}

// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test]
//     fn test_counter() {
//         use stylus_sdk::testing::*;
//         let vm = TestVM::default();
//         let mut contract = Counter::from(&vm);
//
//         assert_eq!(U256::ZERO, contract.number());
//
//         contract.increment();
//         assert_eq!(U256::from(1), contract.number());
//
//         contract.add_number(U256::from(3));
//         assert_eq!(U256::from(4), contract.number());
//
//         contract.mul_number(U256::from(2));
//         assert_eq!(U256::from(8), contract.number());
//
//         contract.set_number(U256::from(100));
//         assert_eq!(U256::from(100), contract.number());
//
//         // Override the msg value for future contract method invocations.
//         vm.set_value(U256::from(2));
//
//         contract.add_from_msg_value();
//         assert_eq!(U256::from(102), contract.number());
//     }
// }
