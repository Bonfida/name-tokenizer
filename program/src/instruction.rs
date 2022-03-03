pub use crate::processor::{create_central_state, create_nft, redeem_nft, withdraw_tokens};
use {
    bonfida_utils::InstructionsAccount,
    borsh::{BorshDeserialize, BorshSerialize},
    num_derive::FromPrimitive,
    solana_program::{instruction::Instruction, pubkey::Pubkey},
};
#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// Create central state
    ///
    /// | Index | Writable | Signer | Description                |
    /// | ------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The central state account  |
    /// | 1     | ✅        | ✅      | The fee payer              |
    /// | 2     | ❌        | ❌      | The system program account |
    CreateCentralState,
    /// Tokenize a domain name
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The mint of the NFT                  |
    /// | 1     | ✅        | ❌      | The NFT token destination            |
    /// | 2     | ✅        | ❌      | The domain name account              |
    /// | 3     | ✅        | ❌      | The NFT record account               |
    /// | 4     | ✅        | ✅      | The domain name owner                |
    /// | 5     | ✅        | ❌      | The metadata account                 |
    /// | 6     | ❌        | ❌      | The central state account            |
    /// | 7     | ❌        | ❌      | The SPL token program account        |
    /// | 8     | ❌        | ❌      | The metadata program account         |
    /// | 9     | ❌        | ❌      | The system program account           |
    /// | 10    | ❌        | ❌      | The SPL name service program account |
    CreateNft,
    /// Redeem a tokenized domain name
    ///
    /// | Index | Writable | Signer | Description                               |
    /// | --------------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The mint of the NFT                       |
    /// | 1     | ✅        | ❌      | The current token account holding the NFT |
    /// | 2     | ✅        | ✅      | The NFT owner account                     |
    /// | 3     | ✅        | ❌      | The NFT record account                    |
    /// | 4     | ✅        | ❌      | The domain name account                   |
    /// | 5     | ❌        | ❌      | The SPL token program account             |
    /// | 6     | ❌        | ❌      | The SPL name service program account      |
    RedeemNft,
    /// Withdraw funds that have been sent to the escrow
    /// while the domain was tokenized
    ///
    /// | Index | Writable | Signer | Description                                |
    /// | ---------------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The token account holding the NFT          |
    /// | 1     | ✅        | ✅      | The owner of the NFT token account         |
    /// | 2     | ✅        | ❌      | The NFT record account                     |
    /// | 3     | ✅        | ❌      | The destination for tokens being withdrawn |
    /// | 4     | ✅        | ❌      | The source for tokens being withdrawn      |
    /// | 5     | ❌        | ❌      | The SPL token program account              |
    WithdrawTokens,
}
#[allow(missing_docs)]
pub fn create_central_state(
    accounts: create_central_state::Accounts<Pubkey>,
    params: create_central_state::Params,
) -> Instruction {
    accounts.get_instruction(
        crate::ID,
        ProgramInstruction::CreateCentralState as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn create_nft(
    accounts: create_nft::Accounts<Pubkey>,
    params: create_nft::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::CreateNft as u8, params)
}
#[allow(missing_docs)]
pub fn redeem_nft(
    accounts: redeem_nft::Accounts<Pubkey>,
    params: redeem_nft::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::RedeemNft as u8, params)
}
#[allow(missing_docs)]
pub fn withdraw_tokens(
    accounts: withdraw_tokens::Accounts<Pubkey>,
    params: withdraw_tokens::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::WithdrawTokens as u8, params)
}
