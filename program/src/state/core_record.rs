use bonfida_utils::BorshSize;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::error::TokenizerError;

use super::Tag;

#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
#[allow(missing_docs)]
pub struct CoreRecord {
    /// Tag
    pub tag: Tag,

    /// Nonce
    pub nonce: u8,

    /// Name account of the record
    pub name_account: Pubkey,

    /// Record owner
    pub owner: Pubkey,

    /// MPL Core asset key
    pub core_asset: Pubkey,
}

#[allow(missing_docs)]
impl CoreRecord {
    pub const SEED: &'static [u8; 11] = b"core_record";

    pub fn new(nonce: u8, owner: Pubkey, name_account: Pubkey, core_asset: Pubkey) -> Self {
        Self {
            tag: Tag::ActiveCoreRecord,
            nonce,
            owner,
            name_account,
            core_asset,
        }
    }

    pub fn find_key(name_account: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[CoreRecord::SEED, &name_account.to_bytes()];
        Pubkey::find_program_address(seeds, program_id)
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(a: &AccountInfo, tag: Tag) -> Result<CoreRecord, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != tag as u8 {
            return Err(TokenizerError::DataTypeMismatch.into());
        }
        let result = CoreRecord::deserialize(&mut data)?;
        Ok(result)
    }

    pub fn is_active(&self) -> bool {
        self.tag == Tag::ActiveCoreRecord
    }
}
