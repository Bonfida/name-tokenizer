use {
    bonfida_utils::checks::check_account_owner,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, hash::hashv, msg,
        program_error::ProgramError,
    },
    spl_name_service::state::{get_seeds_and_key, HASH_PREFIX},
};

use crate::state::ROOT_DOMAIN_ACCOUNT;

pub fn check_name(name: &str, account: &AccountInfo) -> ProgramResult {
    check_account_owner(account, &spl_name_service::ID)?;

    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec();

    if hashed_name.len() != 32 {
        msg!("Invalid seed length");
        return Err(ProgramError::InvalidArgument);
    }

    let (name_account_key, _) = get_seeds_and_key(
        &spl_name_service::ID,
        hashed_name,
        None,
        Some(&ROOT_DOMAIN_ACCOUNT),
    );

    if &name_account_key != account.key {
        msg!("Provided wrong name account");
        #[cfg(not(feature = "devnet"))]
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}
