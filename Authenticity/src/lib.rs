// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
mod utility;
mod verify_signature;

use crate::utility::{EriError::*, *};
use crate::verify_signature::verify;
use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes};
use core::convert::Into;
use stylus_sdk::abi::Bytes;
use stylus_sdk::{alloy_primitives::U256, evm, prelude::*};

sol_interface! {
    interface IEri {
         function createItem(
            address user,
            string calldata name,
            string calldata unique_id,
            string calldata serial,
            uint256 date,
            address owner,
            string[] memory metadata,
            string calldata manufacturer_name
        ) external;
    }
}

sol_storage! {
    #[entrypoint]
    pub struct Authenticity {
        bytes32 certificate_type_hash;
        bytes32 eip712_domain_type_hash;

        string signing_domain;
        string signature_version;
        address ownership;

        mapping(address => Manufacturer) manufacturers;
        mapping(string => address) names;
    }

    struct Manufacturer {
        string name;
        address manufacturer_address;
    }
}

impl Authenticity {
    fn address_zero_check(&self, caller: Address) -> Result<(), EriError> {
        if caller.is_zero() {
            return Err(AddressZero(ADDRESS_ZERO { zero: caller }));
        }
        Ok(())
    }

    fn is_registered(&self, address: Address) -> Result<(), EriError> {
        if self
            .manufacturers
            .getter(address)
            .manufacturer_address
            .get()
            .is_zero()
        {
            return Err(NotRegistered(NOT_REGISTERED { user: address }));
        }
        Ok(())
    }
}

#[public]
impl Authenticity {
    #[constructor]
    pub fn constructor(&mut self, ownership_addr: Address) -> Result<(), EriError> {
        self.ownership.set(ownership_addr);

        evm::log(ContractCreated {
            contractAddress: self.vm().contract_address(),
            owner: self.vm().tx_origin(),
        });

        Ok(())
    }

    pub fn manufacturer_registers(&mut self, name: String) -> Result<(), EriError> {
        let caller = self.vm().msg_sender();
        self.address_zero_check(caller)?;
        self.is_registered(caller)?;

        if !self
            .manufacturers
            .getter(caller)
            .manufacturer_address
            .get()
            .is_zero()
        {
            return Err(Registered(ALREADY_REGISTERED { user: caller }));
        }

        if name.len() < 2 {
            return Err(InvalidManufacturerName(INVALID_MANUFACTURER_NAME {
                name: name.clone(),
            }));
        }

        if !self.names.get(name.clone()).is_empty() {
            return Err(NameNotAvailable(NAME_NOT_AVAILABLE { name: name.clone() }));
        }

        let mut new_manufacturer = self.manufacturers.setter(caller);
        new_manufacturer.manufacturer_address.set(caller);
        new_manufacturer.name.set_str(&name);

        self.names.setter(name.clone()).set(caller);

        evm::log(ManufacturerRegistered { //to test, I will have to comment this out
            manufacturerAddress: caller,
            manufacturerName: name.clone().parse().unwrap(),
        });

        Ok(())
    }

    fn get_manufacturer_address_by_name(&self, name: String) -> Result<Address, EriError> {
        let address = self.names.get(name);

        if address.is_zero() {
            return Err(DoesNotExist(DOES_NOT_EXIST {}));
        }

        Ok(address)
    }
    fn get_manufacturer(&self, address: Address) -> Result<(String, Address), EriError> {
        if self
            .manufacturers
            .getter(address)
            .manufacturer_address
            .get()
            .is_zero()
        {
            return Err(DoesNotExist(DOES_NOT_EXIST {}));
        }

        let manufacturer = self.manufacturers.getter(address);
        Ok((
            manufacturer.name.get_string(),
            manufacturer.manufacturer_address.get(),
        ))
    }

    fn get_manufacturer_address(&self, address: Address) -> Result<Address, EriError> {
        let manufacturer = self
            .manufacturers
            .getter(address)
            .manufacturer_address
            .get();

        if manufacturer.is_zero() || manufacturer != address {
            return Err(DoesNotExist(DOES_NOT_EXIST {}));
        }

        Ok(manufacturer)
    }

    fn verify_signature(
        &self,
        name: String,
        unique_id: String,
        serial: String,
        date: U256,
        owner: Address,
        metadata_hash: FixedBytes<32>,
        signature: Bytes,
    ) -> Result<bool, EriError> {
        let manufacturer = self.manufacturers.get(owner).manufacturer_address.get();

        if manufacturer.is_zero() || manufacturer != owner {
            return Err(DoesNotExist(DOES_NOT_EXIST {}));
        }

        let result = verify(
            name,
            unique_id,
            serial,
            date,
            owner,
            metadata_hash,
            signature,
        )?;

        Ok(result)
    }

    fn user_claim_ownership(
        &mut self,
        name: String,
        unique_id: String,
        serial: String,
        date: U256,
        owner: Address,
        metadata: Vec<String>,
        metadata_hash: FixedBytes<32>,
        signature: Bytes,
    ) -> Result<(), EriError> {
        let ownership = IEri::new(self.ownership.get());

        let caller = self.vm().msg_sender();

        self.address_zero_check(caller)?;

        let manufacturer = self.manufacturers.get(owner).name.get_string();

        match self.verify_signature(
            name.clone(),
            unique_id.clone(),
            serial.clone(),
            date,
            owner,
            metadata_hash,
            signature,
        ) {
            Ok(_) => Ok(ownership
                .create_item(
                    self,
                    caller,
                    name,
                    unique_id,
                    serial,
                    date,
                    owner,
                    metadata,
                    manufacturer,
                )
                .unwrap()),
            Err(_) => Err(ClaimFailed(CLAIM_FAILED {})),
        }
    }

    fn verify_authenticity(
        &self,
        name: String,
        unique_id: String,
        serial: String,
        date: U256,
        owner: Address,
        metadata_hash: FixedBytes<32>,
        signature: Bytes,
    ) -> Result<(bool, String), EriError> {
        match self.verify_signature(
            name.clone(),
            unique_id.clone(),
            serial.clone(),
            date,
            owner,
            metadata_hash,
            signature,
        ) {
            Ok(is_valid) => Ok((is_valid, self.manufacturers.get(owner).name.get_string())),
            Err(_) => Err(InvalidSignature(INVALID_SIGNATURE {})),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::string::ToString;
    use stylus_sdk::console;
    use stylus_sdk::testing::*;

    #[test]
    fn test_manufacturer_registers() {
        let vm = TestVM::default();
        let mut contract = Authenticity::from(&vm);

        let _result = contract.manufacturer_registers("SAMSUNG".to_string());

        match contract.get_manufacturer_address_by_name(String::from("SAMSUNG")) {
            Ok(manufacturer_address) => match contract.get_manufacturer(manufacturer_address) {
                Ok(manu) => {
                    assert_eq!(manu.0, String::from("SAMSUNG"));
                }
                _ => console!("Error!"),
            },
            _ => console!("Error!"),
        }
    }
}
