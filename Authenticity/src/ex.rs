use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::{sol, SolError, SolCall};
use stylus_sdk::{contract, evm, msg, prelude::*};
use alloy_primitives::utils::keccak256;

// Define custom errors
#[derive(SolError)]
pub enum EriError {
    AddressZero(ADDRESS_ZERO),
    AlreadyRegistered(ALREADY_REGISTERED),
    InvalidManufacturerName(INVALID_MANUFACTURER_NAME),
    NameNotAvailable(NAME_NOT_AVAILABLE),
    DoesNotExist(DOES_NOT_EXIST),
    InvalidSignature(INVALID_SIGNATURE),
}

// Define Solidity-like error structs
sol! {
    error ADDRESS_ZERO(address zero);
    error ALREADY_REGISTERED(address user);
    error INVALID_MANUFACTURER_NAME(string name);
    error NAME_NOT_AVAILABLE(string name);
    error DOES_NOT_EXIST();
    error INVALID_SIGNATURE();
}

// Define IEri interface
sol_interface! {
    interface IEri {
        function createItem(address user, Certificate certificate, string manufacturerName) external;
        struct Certificate {
            string name;
            string uniqueId;
            string serial;
            uint256 date;
            address owner;
            bytes32 metadataHash;
            string[] metadata;
        }
        struct Manufacturer {
            address manufacturerAddress;
            string name;
        }
    }
}

// Define events
sol! {
    event ManufacturerRegistered(address indexed manufacturerAddress, string indexed manufacturerName);
    event ContractCreated(address indexed contractAddress, address indexed owner);
}

// Define storage layout
sol_storage! {
    #[entrypoint]
    pub struct Authenticity {
        #[immutable]
        bytes32 certificate_type_hash;
        #[immutable]
        address ownership;
        mapping(address => Manufacturer) manufacturers;
        mapping(string => address) names;
    }
}

// EIP-712 domain struct
sol! {
    struct EIP712Domain {
        string name;
        string version;
        uint256 chainId;
        address verifyingContract;
    }
}

const EIP712_DOMAIN_TYPE_HASH: [u8; 32] = keccak256(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)");

// External methods
#[external]
impl Authenticity {
    /// Initializes the contract with the ownership address and EIP-712 parameters.
    #[constructor]
    pub fn new(
        ownership_addr: Address,
        certificate: String,
        signing_domain: String,
        signature_version: String,
    ) -> Result<(), EriError> {
        let mut this = Self::deploy()?;
        this.ownership.set(ownership_addr);
        this.certificate_type_hash.set(keccak256(certificate.as_bytes()));

        // Emit ContractCreated event
        evm::log(ContractCreated {
            contractAddress: contract::address(),
            owner: msg::sender(),
        });

        Ok(())
    }
    
    

    /// Registers a manufacturer with a unique name.
    pub fn manufacturer_registers(&mut self, name: String) -> Result<(), EriError> {
        let user = msg::sender();
        self.address_zero_check(user)?;

        if self.is_registered(user) {
            return Err(EriError::AlreadyRegistered(ALREADY_REGISTERED { user }));
        }

        if name.len() < 2 {
            return Err(EriError::InvalidManufacturerName(INVALID_MANUFACTURER_NAME { name: name.clone() }));
        }

        if self.names.get(&name).is_some() {
            return Err(EriError::NameNotAvailable(NAME_NOT_AVAILABLE { name }));
        }

        let mut new_manufacturer = self.manufacturers.setter(user);
        new_manufacturer.manufacturer_address.set(user);
        new_manufacturer.name.set_str(&name);

        self.names.setter(name.clone()).set(user);

        evm::log(ManufacturerRegistered {
            manufacturerAddress: user,
            manufacturerName: name,
        });

        Ok(())
    }

    /// Returns the manufacturer address by name.
    pub fn get_manufacturer_by_name(&self, manufacturer_name: String) -> Result<Address, EriError> {
        let manufacturer = self.names.get(&manufacturer_name).ok_or(EriError::DoesNotExist(DOES_NOT_EXIST {}))?;
        Ok(manufacturer)
    }

    /// Returns the manufacturer details by address.
    pub fn get_manufacturer(&self, user_address: Address) -> Result<IEri::Manufacturer, EriError> {
        let manufacturer = self.manufacturers.get(user_address);
        if manufacturer.manufacturer_address.get().is_zero() {
            return Err(EriError::DoesNotExist(DOES_NOT_EXIST {}));
        }
        Ok(IEri::Manufacturer {
            manufacturerAddress: manufacturer.manufacturer_address.get(),
            name: manufacturer.name.get_string(),
        })
    }

    /// Returns the manufacturer address for verification.
    pub fn get_manufacturer_address(&self, expected_manufacturer: Address) -> Result<Address, EriError> {
        let manufacturer = self.manufacturers.get(expected_manufacturer);
        let manufacturer_address = manufacturer.manufacturer_address.get();
        if manufacturer_address.is_zero() || expected_manufacturer != manufacturer_address {
            return Err(EriError::DoesNotExist(DOES_NOT_EXIST {}));
        }
        Ok(manufacturer_address)
    }

    /// Verifies an EIP-712 signature for a certificate.
    pub fn verify_signature(&self, certificate: IEri::Certificate, signature: Vec<u8>) -> Result<bool, EriError> {
        let struct_hash = keccak256(
            alloy_sol_types::abi::encode(&[
                self.certificate_type_hash.get().into(),
                keccak256(certificate.name.as_bytes()).into(),
                keccak256(certificate.uniqueId.as_bytes()).into(),
                keccak256(certificate.serial.as_bytes()).into(),
                certificate.date.into(),
                certificate.owner.into(),
                certificate.metadataHash.into(),
            ])
        );

        let digest = self.hash_typed_data_v4(struct_hash)?;
        let signer = digest.recover(signature).map_err(|_| EriError::InvalidSignature(INVALID_SIGNATURE {}))?;

        let manufacturer = self.get_manufacturer_address(certificate.owner)?;
        if signer != manufacturer {
            return Err(EriError::InvalidSignature(INVALID_SIGNATURE {}));
        }

        Ok(true)
    }

    /// Hashes a struct hash using EIP-712 rules.
    pub fn hash_typed_data_v4(&self, struct_hash: FixedBytes<32>) -> Result<FixedBytes<32>, EriError> {
        let domain_separator = keccak256(
            alloy_sol_types::abi::encode(&[
                EIP712_DOMAIN_TYPE_HASH.into(),
                keccak256("CertificateAuth".as_bytes()).into(),
                keccak256("1".as_bytes()).into(),
                U256::from(contract::chain_id()).into(),
                contract::address().into(),
            ])
        );

        let digest = keccak256(
            alloy_sol_types::abi::encode(&[
                alloy_primitives::hex::decode("1901").unwrap().into(),
                domain_separator.into(),
                struct_hash.into(),
            ])
        );

        Ok(digest)
    }

    /// Allows a user to claim ownership of a certificate after signature verification.
    pub fn user_claim_ownership(&mut self, certificate: IEri::Certificate, signature: Vec<u8>) -> Result<(), EriError> {
        self.address_zero_check(msg::sender())?;
        self.verify_signature(certificate.clone(), signature)?;

        let manufacturer_name = self.manufacturers.get(certificate.owner).name.get_string();
        if manufacturer_name.is_empty() {
            return Err(EriError::DoesNotExist(DOES_NOT_EXIST {}));
        }

        IEri::createItemCall {
            user: msg::sender(),
            certificate,
            manufacturerName: manufacturer_name,
        }
            .call(self.ownership.get())
            .map_err(|_| EriError::InvalidSignature(INVALID_SIGNATURE {}))?;

        Ok(())
    }

    /// Verifies the authenticity of a certificate and returns the manufacturer name.
    pub fn verify_authenticity(&self, certificate: IEri::Certificate, signature: Vec<u8>) -> Result<(bool, String), EriError> {
        let is_valid = self.verify_signature(certificate.clone(), signature)?;
        let manufacturer_name = self.manufacturers.get(certificate.owner).name.get_string();
        if manufacturer_name.is_empty() {
            return Err(EriError::DoesNotExist(DOES_NOT_EXIST {}));
        }
        Ok((is_valid, manufacturer_name))
    }

    /// Checks if a user is a registered manufacturer.
    fn is_registered(&self, user: Address) -> bool {
        !self.manufacturers.get(user).manufacturer_address.get().is_zero()
    }

    /// Checks if an address is zero.
    fn address_zero_check(&self, addr: Address) -> Result<(), EriError> {
        if addr.is_zero() {
            return Err(EriError::AddressZero(ADDRESS_ZERO { zero: addr }));
        }
        Ok(())
    }
}