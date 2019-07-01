use std::convert::{TryFrom, TryInto};

use failure_ext::prelude::*;
use types::{
    account_address::AccountAddress, account_config::AccountResource,
    account_state_blob::AccountStateBlob,
};

/// Converts hex representation of an address into binary.
pub fn address_from_strings(data: &str) -> Result<AccountAddress> {
    let account_vec: Vec<u8> = hex::decode(data.parse::<String>()?)?;

    let account = match AccountAddress::try_from(&account_vec[..]) {
        Ok(address) => address,
        Err(error) => bail!(
            "The address {:?} is invalid, error: {:?}",
            &account_vec,
            error,
        ),
    };

    Ok(account)
}

pub fn get_account_resource_or_default(
    account_state: &Option<AccountStateBlob>,
) -> Result<AccountResource> {
    match account_state {
        Some(blob) => {
            let account_btree = blob.try_into()?;
            AccountResource::make_from(&account_btree)
        }
        None => Ok(AccountResource::default()),
    }
}
