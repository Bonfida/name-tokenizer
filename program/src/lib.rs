use solana_program::declare_id;
#[doc(hidden)]
pub mod entrypoint;
#[doc(hidden)]
pub mod error;
/// Program instructions and their CPI-compatible bindings
pub mod instruction;
/// Describes the different data structres that the program uses to encode state
pub mod state;

#[doc(hidden)]
pub(crate) mod processor;
pub(crate) mod utils;

#[allow(missing_docs)]
pub mod cpi;

declare_id!("8cLRM8yC7gdtEPLxjSkekxXBQHX35hnb4SVbf8RhDSCK");
