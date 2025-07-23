#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

mod utility;

#[macro_use]
extern crate alloc;

use crate::utility::EriError::*;
use crate::utility::*; //{AuthenticitySet, ContractCreated, EriError, ADDRESS_ZERO, ONLY_OWNER};
use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::Address;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Ownership {

        address authenticity;

        address owner;

        mapping(string => UserProfile) users;

        mapping(address => string) usernames;

        mapping(string => address) owners;

        mapping(address => mapping(string => Item)) owned_items;

        mapping(address => Item[]) my_items;

        mapping(bytes32 => address) temp;

        mapping(bytes32 => mapping(address => Item)) temp_owners;
    }

    struct UserProfile {
        address user_address;
        string username;
        bool registered;
        uint256 registered_at;
    }

    struct Item {
        string name;
        string item_id;
        string serial;
        uint256 date;
        address owner;
        string manufacturer;
        string[] metadata;
    }
}

impl Ownership {
    fn address_zero_check(&self, caller: Address) -> Result<(), EriError> {
        // let caller = self.vm().msg_sender();

        if caller.is_zero() {
            return Err(AddressZero(ADDRESS_ZERO { zero: caller }));
        }
        Ok(())
    }
}

#[public]
impl Ownership {
    #[constructor]
    fn constructor(&mut self, owner: Address) -> Result<(), EriError> {
        self.address_zero_check(owner)?;

        self.owner.set(owner);

        stylus_sdk::evm::log(ContractCreated {
            contractAddress: self.vm().contract_address(),
            owner,
        });

        Ok(())
    }

    fn set_authenticity(&mut self, authenticity_address: Address) -> Result<(), EriError> {
        self.address_zero_check(authenticity_address)?;
        let caller = self.vm().msg_sender();
        if caller != self.owner.get() {
            //ONLY OWNER
            return Err(OnlyOwner(ONLY_OWNER { owner: caller }));
        }

        self.owner.set(authenticity_address);

        stylus_sdk::evm::log(AuthenticitySet {
            authenticityAddress: authenticity_address,
        });

        Ok(())
    }

    //
    //     function userRegisters(
    //     string calldata username
    //     ) external addressZeroCheck(msg.sender) isAuthenticitySet {
    //     address userAddress = msg.sender;
    //     users._userRegisters(usernames, userAddress, username);
    //     emit UserRegistered(userAddress, username);
    // }

    // if (bytes(username).length < 3) {
    // revert EriErrors.USERNAME_MUST_BE_AT_LEAST_3_LETTERS();
    // }
    // //reverts if username is already used by someone else
    // if (isRegistered(users, username)) {
    // //no duplicate username and address
    // revert EriErrors.NAME_NOT_AVAILABLE(username);
    // }
    // //reverts if wallet address has already registered
    // if (isNotEmpty(usernames[userAddress])) {
    // revert EriErrors.ALREADY_REGISTERED(userAddress);
    // }
    //
    // IEri.UserProfile storage _user = users[username];
    // _user.userAddress = userAddress;
    // _user.username = username;
    // _user.isRegistered = true;
    // _user.registeredAt = block.timestamp;
    //
    // //save a username with a user address, mostly for when using connect wallet
    // usernames[userAddress] = username;

    fn user_registers(&mut self, username: String) -> Result<(), EriError> {
        let caller = self.vm().msg_sender();
        self.address_zero_check(caller)?;

        let time = self.vm().block_timestamp();

        if username.len() < 3 {
            return Err(BadUsername(USERNAME_MUST_BE_AT_LEAST_3_LETTERS {}));
        }

        let mut user = self.users.setter(username.clone());

        if user.registered.get() {
            return Err(NotAvailable(NAME_NOT_AVAILABLE {
                username: username.clone(),
            }));
        }

        let mut fetched_username = self.usernames.setter(caller);
        if !fetched_username.get_string().is_empty() {
            return Err(Registered(ALREADY_REGISTERED { caller }));
        }

        user.user_address.set(caller);
        user.username.set_str(username.clone());
        user.registered.set(true);
        user.registered_at.set(U256::from(time));

        fetched_username.set_str(username.clone());

        stylus_sdk::evm::log(UserRegistered {
            userAddress: caller,
            username: username.parse().unwrap(),
        });

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_counter() {
        // use stylus_sdk::testing::*;
        // let vm = TestVM::default();
        // let mut contract = Counter::from(&vm);
        //
        // assert_eq!(U256::ZERO, contract.number());
        //
        // contract.increment();
        // assert_eq!(U256::from(1), contract.number());
        //
        // contract.add_number(U256::from(3));
        // assert_eq!(U256::from(4), contract.number());
        //
        // contract.mul_number(U256::from(2));
        // assert_eq!(U256::from(8), contract.number());
        //
        // contract.set_number(U256::from(100));
        // assert_eq!(U256::from(100), contract.number());
        //
        // // Override the msg value for future contract method invocations.
        // vm.set_value(U256::from(2));
        //
        // contract.add_from_msg_value();
        // assert_eq!(U256::from(102), contract.number());
    }
}
