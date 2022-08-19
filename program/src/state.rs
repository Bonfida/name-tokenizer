use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_token_metadata::state::Creator,
    solana_program::{pubkey, pubkey::Pubkey},
};

mod central_state;
mod nft_record;

pub use central_state::CentralState;
pub use nft_record::NftRecord;

pub const ROOT_DOMAIN_ACCOUNT: Pubkey = pubkey!("58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx");

pub const MINT_PREFIX: &[u8; 14] = b"tokenized_name";

pub const SELLER_BASIS: u16 = 500;

pub const META_SYMBOL: &str = ".sol";

pub const CREATOR_KEY: Pubkey = pubkey!("94xt1Eyc56YDU6MtV7KsG8xfeRqd7z272g14tBHztnUM");

pub const CREATOR_FEE: Creator = Creator {
    address: CREATOR_KEY,
    verified: false,
    share: 100,
};

pub const COLLECTION_PREFIX: &[u8; 10] = b"collection";

pub const COLLECTION_NAME: &str = "Solana name service collection";

pub const COLLECTION_URI: &str =
    "https://cloudflare-ipfs.com/ipfs/QmPeTioTicb19seM6itP8KD39syNZVJS2KHXNkxauSGXAJ";

pub const METADATA_SIGNER: Pubkey = pubkey!("ARy9ZzW9qFCb8c8Lxi4NCph1TRNabUaMH5tj4e5pqwHb");

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    CentralState,
    ActiveRecord,
    InactiveRecord,
}
