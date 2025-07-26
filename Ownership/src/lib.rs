#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
mod utility;

use crate::utility::EriError::*;
use crate::utility::*;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes};
use stylus_sdk::{alloy_primitives::U256, prelude::*};

// sol_storage! {
//     #[entrypoint]
//     pub struct Ownership {
//
//         address authenticity;
//
//         address owner;
//
//         mapping(string => UserProfile) users;
//
//         mapping(address => string) usernames;
//
//         mapping(string => address) owners;
//
//         mapping(address => mapping(string => Item)) owned_items;
//
//         mapping(address => Item[]) my_items;
//
//         mapping(bytes32 => address) temp;
//
//         mapping(bytes32 => mapping(address => Item)) temp_owners;
//     }
//
//     struct UserProfile {
//         address user_address;
//         string username;
//         bool registered;
//         uint256 registered_at;
//     }
//
//     struct Item {
//         string name;
//         string item_id;
//         string serial;
//         uint256 date;
//         address owner;
//         string manufacturer;
//         string[] metadata;
//     }
// }

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

    //   struct Certificate {
    //     string name;
    //     string unique_id;
    //     string serial;
    //     uint256 date;
    //     address owner;
    //     bytes32 metadata_hash;
    //     string[] metadata;
    // }
}

impl Ownership {
    fn address_zero_check(&self, caller: Address) -> Result<(), EriError> {
        if caller.is_zero() {
            return Err(AddressZero(ADDRESS_ZERO { zero: caller }));
        }
        Ok(())
    }

    fn is_authenticity_set(&self) -> Result<(), EriError> {
        if self.authenticity.get().is_zero() {
            return Err(AuthenticityNotSet(AUTHENTICITY_NOT_SET {}));
        }

        Ok(())
    }

    fn is_registered(&self, address: Address) -> Result<(), EriError> {
        if !self
            .users
            .get(self.usernames.get(address).get_string())
            .registered
            .get()
        {
            return Err(NotRegistered(NOT_REGISTERED { user: address }));
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
        self.is_authenticity_set()?;
        self.address_zero_check(authenticity_address)?;
        let caller = self.vm().msg_sender();
        //ONLY OWNER
        if caller != self.owner.get() {
            return Err(OnlyOwner(ONLY_OWNER { owner: caller }));
        }

        self.owner.set(authenticity_address);

        stylus_sdk::evm::log(AuthenticitySet {
            authenticityAddress: authenticity_address,
        });

        Ok(())
    }
    fn user_registers(&mut self, username: String) -> Result<(), EriError> {
        let caller = self.vm().msg_sender();

        self.is_authenticity_set()?;
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
    fn get_user(&self, user_address: Address) -> Result<(Address, String, bool, U256), EriError> {
        self.is_authenticity_set()?;
        let username = self.usernames.get(user_address);
        let user = self.users.get(username.get_string());

        if user.user_address.get().is_zero() {
            return Err(NotExist(USER_DOES_NOT_EXIST { user: user_address }));
        }

        Ok((
            user.user_address.get(),
            user.username.get_string(),
            user.registered.get(),
            user.registered_at.get(),
        ))
    }

    fn create_item(
        &mut self,
        user: Address,
        name: String,
        unique_id: String,
        serial: String,
        date: U256,
        owner: Address,
        // metadata_hash: FixedBytes<32>,
        metadata: Vec<String>,
        manufacturer_name: String,
    ) -> Result<(), EriError> {
        let caller = self.vm().msg_sender();

        self.is_authenticity_set()?;
        self.address_zero_check(caller)?;
        self.address_zero_check(user)?;
        self.is_registered(user)?;

        if caller != self.authenticity.get() {
            //making sure only Authenticity can call this function
            return Err(Unauthorized(UNAUTHORIZED { caller }));
        }
        if owner.is_zero() {
            return Err(AddressZero(ADDRESS_ZERO { zero: owner }));
        }

        if !self.owners.get(unique_id.clone()).is_zero() {
            return Err(AlreadyClaimed(ITEM_CLAIMED_ALREADY {
                itemId: unique_id.clone(),
            }));
        }

        //======== GENERAL ITEMS ==========
        let mut user_item = self.owned_items.setter(user);
        let mut item = user_item.setter(unique_id.clone());

        item.item_id.set_str(unique_id.clone());
        item.owner.set(user);
        item.name.set_str(name.clone());
        item.date.set(date);
        item.manufacturer.set_str(manufacturer_name.clone());
        item.serial.set_str(serial.clone());

        //======== PERSONAL ITEM =============
        let mut my_items_vec = self.my_items.setter(user);
        let mut new_item = my_items_vec.grow();

        new_item.item_id.set_str(unique_id.clone());
        new_item.owner.set(user);
        new_item.name.set_str(name);
        new_item.date.set(date);
        new_item.manufacturer.set_str(manufacturer_name);
        new_item.serial.set_str(serial);

        for meta in metadata {
            let mut guard = item.metadata.grow();
            guard.set_str(meta.clone());

            // Adds a new StorageString slot and returns a guard
            let mut guard = new_item.metadata.grow();
            guard.set_str(meta);
        }

        // item id to a user address
        self.owners.setter(unique_id.clone()).set(user);

        stylus_sdk::evm::log(ItemCreated {
            itemId: unique_id.parse().unwrap(),
            owner: user,
        });

        Ok(())
    }

    // if (users[usernames[user]].userAddress == address(0)) {
    // revert EriErrors.USER_DOES_NOT_EXIST(user);
    // }
    //
    // IEri.Item[] memory itemList = myItems[user];
    //
    // // Count valid items
    // uint256 validCount = 0;
    // for (uint256 i = 0; i < itemList.length; i++) {
    // if (ownedItems[user][itemList[i].itemId].owner != address(0)) {
    // validCount++;
    // }
    // }
    //
    // if (validCount == 0) {
    // return new IEri.Item[](0);
    // }
    //
    // // Allocate and populate array
    // IEri.Item[] memory newItemList = new IEri.Item[](validCount);
    // for (uint256 i = 0; i < itemList.length; i++) {
    // if (ownedItems[user][itemList[i].itemId].owner != address(0)) {
    // newItemList[validCount - 1] = ownedItems[user][
    // itemList[i].itemId
    // ];
    // validCount--;
    // }
    // }
    //
    // return newItemList;

    fn get_all_my_items(
        &self,
    ) -> Result<Vec<(String, String, String, U256, Address, String, Vec<String>)>, EriError> {
        self.is_authenticity_set()?;

        let caller = self.vm().msg_sender();

        if self
            .users
            .get(self.usernames.get(caller).get_string())
            .user_address
            .get()
            .is_zero()
        {
            return Err(NotExist(USER_DOES_NOT_EXIST { user: caller }));
        }

        let item_list = self.my_items.get(caller);

        let mut new_list = Vec::new();

        for i in 0..item_list.len() {
            let item_guard = self.owned_items.get(caller);
            let owned_item = item_guard.get(item_list.get(i).unwrap().item_id.get_string());

            if !owned_item.owner.get().is_zero() {
                let mut meta = Vec::new();

                for i in 0..owned_item.metadata.len() {
                    meta.push(owned_item.metadata.get(i).unwrap().get_string())
                }

                new_list.push((
                    owned_item.name.get_string(),
                    owned_item.item_id.get_string(),
                    owned_item.serial.get_string(),
                    owned_item.date.get(),
                    owned_item.owner.get(),
                    owned_item.manufacturer.get_string(),
                    meta,
                ))
            }
        }
        Ok(new_list)
    }
}
