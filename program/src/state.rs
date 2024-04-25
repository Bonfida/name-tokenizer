use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_token_metadata::types::Creator,
    solana_program::{pubkey, pubkey::Pubkey},
};

mod central_state;
mod core_record;
mod nft_record;

pub use central_state::CentralState;
pub use core_record::CoreRecord;
pub use nft_record::NftRecord;

pub const ROOT_DOMAIN_ACCOUNT: Pubkey = pubkey!("58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx");

pub const MINT_PREFIX: &[u8; 14] = b"tokenized_name";

pub const CORE_ASSET_PREFIX: &[u8; 25] = b"tokenized_name_core_asset";

pub const SELLER_BASIS: u16 = 500;

pub const META_SYMBOL: &str = ".sol";

pub const CREATOR_KEY: Pubkey = pubkey!("5D2zKog251d6KPCyFyLMt3KroWwXXPWSgTPyhV22K2gR");

pub const CREATOR_FEE: Creator = Creator {
    address: CREATOR_KEY,
    verified: false,
    share: 100,
};

pub const COLLECTION_PREFIX: &[u8; 10] = b"collection";

pub const CORE_COLLECTION_PREFIX: &[u8; 15] = b"core_collection";

pub const COLLECTION_NAME: &str = "Solana name service collection";

pub const COLLECTION_CORE_NAME: &str = "Solana Name Service (SNS)";

// TODO: Change
pub const COLLECTION_CORE_URI: &str = "";

pub const COLLECTION_URI: &str =
    "https://cloudflare-ipfs.com/ipfs/QmPeTioTicb19seM6itP8KD39syNZVJS2KHXNkxauSGXAJ";

pub const METADATA_SIGNER: Pubkey = pubkey!("Es33LnWSTZ9GbW6yBaRkSLUaFibVd7iS54e4AvBg76LX");

// TODO: Change
pub const METADATA_CORE_SIGNER: Pubkey = pubkey!("Es33LnWSTZ9GbW6yBaRkSLUaFibVd7iS54e4AvBg76LX");

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    CentralState,
    ActiveRecord,
    InactiveRecord,
    ActiveCoreRecord,
    InactiveCoreRecord,
}
