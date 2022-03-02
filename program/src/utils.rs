use {
    bonfida_utils::checks::check_account_owner,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
        program_pack::Pack, pubkey::Pubkey,
    },
    spl_token::state::Account,
};

#[allow(dead_code)]
pub fn assert_uninitialized(account: &AccountInfo) -> ProgramResult {
    if !account.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(())
}

#[allow(dead_code)]
pub fn get_mint_from_account_info(account: &AccountInfo) -> Result<Pubkey, ProgramError> {
    let token_acc = Account::unpack_from_slice(&account.data.borrow())?;
    msg!("{:?}", token_acc);
    Ok(token_acc.mint)
}
