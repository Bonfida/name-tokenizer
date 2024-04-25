use crate::{error::TokenizerError, processor::Processor};

use {
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo, decode_error::DecodeError, entrypoint::ProgramResult, msg,
        program_error::PrintProgramError, pubkey::Pubkey,
    },
};

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

/// The entrypoint to the program
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint");
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<TokenizerError>();
        return Err(error);
    }
    Ok(())
}

impl PrintProgramError for TokenizerError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            TokenizerError::AlreadyInitialized => {
                msg!("Error: This account is already initialized")
            }
            TokenizerError::DataTypeMismatch => msg!("Error: Data type mismatch"),
            TokenizerError::WrongOwner => msg!("Error: Wrong account owner"),
            TokenizerError::Uninitialized => msg!("Error: Account is uninitialized"),
            TokenizerError::InvalidCoreAssetState => msg!("Error: Invalid Core Asset state"),
            TokenizerError::CoreAssetOwnerMismatch => msg!("Error: Core Asset owner mismatch"),
            TokenizerError::CoreAssetMistmatch => msg!("Error: Core Asset mismatch"),
        }
    }
}
