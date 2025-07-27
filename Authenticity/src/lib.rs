#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

mod utility;

#[macro_use]
extern crate alloc;

use crate::utility::{EriError::*, *};
use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes};
use alloy_sol_types::{sol, SolValue};
use core::convert::Into;
use ethers::prelude::erc::Metadata;
use stylus_sdk::{alloy_primitives::U256, crypto::keccak, evm, prelude::*};

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

        // IEri OWNERSHIP; this will be the ownership contract interface to use to initialize it

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
    pub fn constructor(
        &mut self,
        ownership_addr: Address,
        certificate: String,
        signing_domain: String,
        signature_version: String,
    ) -> Result<(), EriError> {
        self.ownership.set(ownership_addr);
        self.signature_version.set_str(signature_version);
        self.signing_domain.set_str(signing_domain);

        self.certificate_type_hash
            .set(keccak(certificate.as_bytes()));

        self.eip712_domain_type_hash.set(keccak(
            b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        ));

        evm::log(ContractCreated {
            contractAddress: self.vm().contract_address(),
            owner: self.vm().msg_sender(),
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

        evm::log(ManufacturerRegistered {
            manufacturerAddress: caller,
            manufacturerName: name.clone().parse().unwrap(),
        });

        Ok(())
    }

    fn get_manufacturer_by_name(&self, name: String) -> Result<Address, EriError> {
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

    // struct Certificate {
    //     string name;
    //     string uniqueId;
    //     string serial;
    //     uint256 date;
    //     address owner;
    //     bytes32 metadataHash;
    //     string[] metadata;
    // }

    pub fn verify_signature(
        &self,
        name: String,
        unique_id: String,
        serial: String,
        date: U256,
        owner: Address,
        metadata_hash: FixedBytes<32>,
        signature: Vec<u8>,
    ) -> Result<bool, EriError> {
        let certificate = (name, unique_id, serial, date, owner, metadata_hash);
        type Certificate = (String, String, String, U256, Address, FixedBytes<32>);
        let encoded = Certificate::abi_encode_sequence(&certificate);
        let item_hash: FixedBytes<32> = keccak(encoded).into();


        let digest = self.vm().hash_typed_data_v4(struct_hash)?;
        let signer = digest
            .recover(signature)
            .map_err(|_| EriError::InvalidSignature(INVALID_SIGNATURE {}))?;

        let manufacturer = self.get_manufacturer_address(certificate.owner)?;
        if signer != manufacturer {
            return Err(EriError::InvalidSignature(INVALID_SIGNATURE {}));
        }

        Ok(true)
    }
}
