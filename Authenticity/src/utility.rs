use alloy_sol_types::sol;
use stylus_sdk::prelude::SolidityError;


sol! {

    error ADDRESS_ZERO(address zero);
    error NOT_REGISTERED(address user);
    error ALREADY_REGISTERED(address user);
    error INVALID_MANUFACTURER_NAME(string name);
    error NAME_NOT_AVAILABLE(string name);
    error CLAIM_FAILED();
    error INVALID_SIGNATURE();
    error EC_RECOVER_CALL_ERROR();
    error DOES_NOT_EXIST();


    event ManufacturerRegistered(address indexed manufacturerAddress, string indexed manufacturerName);
    event ContractCreated(address indexed contractAddress, address indexed owner);
}

#[derive(SolidityError)]
pub enum EriError {

    AddressZero(ADDRESS_ZERO),
    NotRegistered(NOT_REGISTERED),
    Registered(ALREADY_REGISTERED),
    InvalidManufacturerName(INVALID_MANUFACTURER_NAME),
    NameNotAvailable(NAME_NOT_AVAILABLE),
    ClaimFailed(CLAIM_FAILED),
    InvalidSignature(INVALID_SIGNATURE),
    ECRecoverError(EC_RECOVER_CALL_ERROR),
    DoesNotExist(DOES_NOT_EXIST)
}

