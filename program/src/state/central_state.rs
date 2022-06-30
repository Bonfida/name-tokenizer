use bonfida_utils::BorshSize;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use super::Tag;

#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
#[allow(missing_docs)]
pub struct CentralState {
    pub tag: Tag,
}

impl CentralState {
    pub fn find_key(program_id: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[&program_id.to_bytes()];
        Pubkey::find_program_address(seeds, program_id)
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }
}
