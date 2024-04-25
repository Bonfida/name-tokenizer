use {
    num_derive::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum TokenizerError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("Data type mismatch")]
    DataTypeMismatch,
    #[error("Wrong account owner")]
    WrongOwner,
    #[error("Account is uninitialized")]
    Uninitialized,
    #[error("Invalid Core Asset state")]
    InvalidCoreAssetState,
    #[error("Core Asset owner mismatch")]
    CoreAssetOwnerMismatch,
    #[error("Core Asset mismatch")]
    CoreAssetMistmatch,
}

impl From<TokenizerError> for ProgramError {
    fn from(e: TokenizerError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TokenizerError {
    fn type_of() -> &'static str {
        "TokenizerError"
    }
}
