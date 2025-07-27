#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
mod utility;

use crate::utility::{EriError::*, *};
use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes};
use alloy_sol_types::SolValue;
use stylus_sdk::{alloy_primitives::U256, crypto::keccak, prelude::*};

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

    #[derive(Erase)]
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
    fn is_item_owner(&self, item_id: String) -> Result<(), EriError> {
        let caller = self.vm().msg_sender();

        if caller != self.owners.get(item_id) {
            return Err(OnlyOwner(ONLY_OWNER { owner: caller }));
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
            user.user_address.get(), //user address
            user.username.get_string(), //user name
            user.registered.get(), //is_registered
            user.registered_at.get(), //reg time
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

        set_item(
            &mut item,
            user,
            name.clone(),
            unique_id.clone(),
            serial.clone(),
            date,
            manufacturer_name.clone(),
        );

        //======== PERSONAL ITEM =============
        let mut my_items_vec = self.my_items.setter(user);
        let mut new_item = my_items_vec.grow();

        set_item(
            &mut new_item,
            user,
            name,
            unique_id.clone(),
            serial,
            date,
            manufacturer_name,
        );

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

                new_list.push(item_tuple(&owned_item, meta))
            }
        }
        Ok(new_list)
    }

    fn generate_change_of_ownership_code(
        &mut self,
        item_id: String,
        temp_owner: Address,
    ) -> Result<(), EriError> {
        let caller = self.vm().msg_sender();

        self.address_zero_check(caller)?;
        self.address_zero_check(temp_owner)?;
        self.is_authenticity_set()?;
        self.is_registered(caller)?;
        self.is_item_owner(item_id.clone())?;

        if caller == temp_owner {
            return Err(CannotGenerate(CANNOT_GENERATE_CODE_FOR_YOURSELF { caller }));
        }

        let mut user_item = self.owned_items.setter(caller);
        let item = user_item.setter(item_id);

        let mut meta = Vec::new();

        for i in 0..item.metadata.len() {
            meta.push(item.metadata.get(i).unwrap().get_string())
        }

        let inner_item = (
            item.name.get_string(),
            item.item_id.get_string(),
            item.serial.get_string(),
            item.date.get(),
            item.owner.get(),
            item.manufacturer.get_string(),
            meta,
        );
        type InnerItemTuple = (String, String, String, U256, Address, String, Vec<String>);
        let encoded = InnerItemTuple::abi_encode_sequence(&inner_item);
        let item_hash: FixedBytes<32> = keccak(encoded).into();

        if !self.temp.get(item_hash).is_zero() {
            return Err(NotClaimed(ITEM_NOT_CLAIMED_YET {}));
        }

        self.temp.setter(item_hash).set(temp_owner);

        let mut owner_guard = self.temp_owners.setter(item_hash);
        let mut t_owner = owner_guard.setter(temp_owner);

        set_item(
            &mut t_owner,
            item.owner.get(),
            item.name.get_string(),
            item.item_id.get_string(),
            item.serial.get_string(),
            item.date.get(),
            item.manufacturer.get_string(),
        );

        stylus_sdk::evm::log(OwnershipCode {
            ownershipCode: item_hash,
            tempOwner: temp_owner,
        });

        Ok(())
    }

    fn new_owner_claim_ownership(&mut self, item_hash: FixedBytes<32>) -> Result<(), EriError> {
        let caller = self.vm().msg_sender();

        self.is_authenticity_set()?;
        self.address_zero_check(caller)?;
        self.is_registered(caller)?;

        let temp_owner = self.temp.get(item_hash);

        let mut user_item = self.temp_owners.setter(item_hash);
        let mut item = user_item.setter(caller);

        let old_owner = item.owner.get();

        if caller != caller || old_owner.is_zero() {
            return Err(Unauthorized(UNAUTHORIZED { caller }));
        }

        item.owner.set(caller); // set the new owner for the item

        //remove the item from old owner's item list
        let mut old_owner_item_list = self.my_items.setter(old_owner);
        for i in 0..old_owner_item_list.len() {
            let mut guard = old_owner_item_list.setter(i).unwrap();
            if guard.item_id.get_string() == item.item_id.get_string() {
                guard.erase();
                break;
            }
        }

        let item_id = item.item_id.get_string();

        self.owned_items.setter(old_owner).delete(item_id.clone()); //delete the item from the old owner mapping

        let mut item_guard = self.owned_items.setter(caller);
        let mut save_item = item_guard.setter(item_id.clone());

        set_item(
            &mut save_item,
            item.owner.get(),
            item.name.get_string(),
            item.item_id.get_string(),
            item.serial.get_string(),
            item.date.get(),
            item.manufacturer.get_string(),
        );

        self.owners.setter(item_id).set(caller);

        let mut item_list = self.my_items.setter(caller);

        let mut guard = item_list.grow();

        set_item(
            &mut guard,
            item.owner.get(),
            item.name.get_string(),
            item.item_id.get_string(),
            item.serial.get_string(),
            item.date.get(),
            item.manufacturer.get_string(),
        );

        self.temp_owners.setter(item_hash).delete(caller);
        self.temp.delete(item_hash);

        stylus_sdk::evm::log(OwnershipClaimed {
            newOwner: caller,
            oldOwner: old_owner,
        });

        Ok(())
    }
    fn get_temp_owner(&self, item_hash: FixedBytes<32>) -> Result<Address, EriError> {
        self.is_authenticity_set()?;

        Ok(self.temp.get(item_hash))
    }

    fn owner_revoke_code(&mut self, item_hash: FixedBytes<32>) -> Result<(), EriError> {
        let caller = self.vm().msg_sender();

        self.is_authenticity_set()?;
        self.address_zero_check(caller)?;
        self.is_registered(caller)?;

        let temp_owner = self.temp.get(item_hash);

        let mut item_guard = self.temp_owners.setter(item_hash);
        let item = item_guard.setter(temp_owner);

        if item.owner.get().is_zero() {
            return Err(DoesNotExist(DOES_NOT_EXIST {}));
        }

        if item.owner.get() != caller {
            return Err(OnlyOwner(ONLY_OWNER { owner: caller }));
        }

        self.temp_owners.setter(item_hash).delete(temp_owner);
        self.temp.delete(item_hash);

        stylus_sdk::evm::log(CodeRevoked {
            itemHash: item_hash,
        });

        Ok(())
    }

    fn get_item(
        &self,
        item_id: String,
    ) -> Result<(String, String, String, U256, Address, String, Vec<String>), EriError> {
        self.is_authenticity_set()?;

        let user = self.owners.get(item_id.clone());

        if user.is_zero() {
            return Err(ItemDoesNotExist(ITEM_DOESNT_EXIST {
                itemId: item_id.clone(),
            }));
        }
        let item_guard = self.owned_items.getter(user);
        let item = item_guard.getter(item_id);

        let mut meta = Vec::new();

        for i in 0..item.metadata.len() {
            meta.push(item.metadata.get(i).unwrap().get_string())
        }

        Ok(item_tuple(&item, meta))
    }

    fn verify_ownership(
        &self,
        item_id: String,
    ) -> Result<(String, String, String, Address), EriError> {
        self.is_authenticity_set()?;

        let user = self.owners.get(item_id.clone());

        if user.is_zero() {
            return Err(ItemDoesNotExist(ITEM_DOESNT_EXIST {
                itemId: item_id.clone(),
            }));
        }

        let item_guard = self.owned_items.getter(user);
        let item = item_guard.getter(item_id);

        Ok((
            item.name.get_string(),
            item.item_id.get_string(),
            self.usernames.getter(item.owner.get()).get_string(),
            item.owner.get(),
        ))
    }

    fn is_owner(&self, user: Address, item_id: String) -> Result<bool, EriError> {
        self.is_authenticity_set()?;

        Ok(self.owned_items.getter(user).getter(item_id).owner.get() == user)
    }
}
