pub use crate::processor::{
    create_collection, create_mint, create_nft, edit_data, redeem_nft, unverify_nft,
    withdraw_tokens,
};
use {
    bonfida_utils::InstructionsAccount,
    borsh::{BorshDeserialize, BorshSerialize},
    num_derive::FromPrimitive,
    solana_program::{instruction::Instruction, pubkey::Pubkey},
};
#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// Create the NFT mint
    ///
    /// | Index | Writable | Signer | Description                   |
    /// | --------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The mint of the NFT           |
    /// | 1     | ✅        | ❌      | The domain name account       |
    /// | 2     | ❌        | ❌      | The central state account     |
    /// | 3     | ❌        | ❌      | The SPL token program account |
    /// | 4     | ❌        | ❌      | The system program account    |
    /// | 5     | ❌        | ❌      | Rent sysvar account           |
    /// | 6     | ❌        | ❌      | Fee payer account             |
    CreateMint,
    /// Create a verified collection
    ///
    /// | Index | Writable | Signer | Description                                                   |
    /// | ----------------------------------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The mint of the collection                                    |
    /// | 1     | ✅        | ❌      |                                                               |
    /// | 2     | ✅        | ❌      | The metadata account                                          |
    /// | 3     | ❌        | ❌      | The central state account                                     |
    /// | 4     | ✅        | ❌      | Token account of the central state to hold the master edition |
    /// | 5     | ❌        | ❌      | The fee payer account                                         |
    /// | 6     | ❌        | ❌      | The SPL token program account                                 |
    /// | 7     | ❌        | ❌      | The metadata program account                                  |
    /// | 8     | ❌        | ❌      | The system program account                                    |
    /// | 9     | ❌        | ❌      | The SPL name service program account                          |
    /// | 10    | ❌        | ❌      |                                                               |
    /// | 11    | ❌        | ❌      | Rent sysvar account                                           |
    CreateCollection,
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
    /// | 6     | ❌        | ❌      | Master edition account               |
    /// | 7     | ❌        | ❌      | Collection                           |
    /// | 8     | ❌        | ❌      | Mint of the collection               |
    /// | 9     | ✅        | ❌      | The central state account            |
    /// | 10    | ✅        | ✅      | The fee payer account                |
    /// | 11    | ❌        | ❌      | The SPL token program account        |
    /// | 12    | ❌        | ❌      | The metadata program account         |
    /// | 13    | ❌        | ❌      | The system program account           |
    /// | 14    | ❌        | ❌      | The SPL name service program account |
    /// | 15    | ❌        | ❌      | Rent sysvar account                  |
    /// | 16    | ❌        | ✅      | The metadata signer                  |
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
    /// | 6     | ❌        | ❌      | The system program account                 |
    WithdrawTokens,
    /// Edit the data registry of a tokenized domain name
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ❌        | ✅      | The NFT owner account                |
    /// | 1     | ❌        | ❌      | The NFT account                      |
    /// | 2     | ❌        | ❌      | The NFT record account               |
    /// | 3     | ✅        | ❌      | The domain name account              |
    /// | 4     | ❌        | ❌      | The SPL token program account        |
    /// | 5     | ❌        | ❌      | The SPL name service program account |
    EditData,
    /// Unverify an NFT
    ///
    /// | Index | Writable | Signer | Description                  |
    /// | -------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The metadata account         |
    /// | 1     | ❌        | ❌      | Master edition account       |
    /// | 2     | ❌        | ❌      | Collection                   |
    /// | 3     | ❌        | ❌      | Mint of the collection       |
    /// | 4     | ✅        | ❌      | The central state account    |
    /// | 5     | ✅        | ✅      | The fee payer account        |
    /// | 6     | ❌        | ❌      | The metadata program account |
    /// | 7     | ❌        | ❌      | The system program account   |
    /// | 8     | ❌        | ❌      | Rent sysvar account          |
    /// | 9     | ❌        | ✅      | The metadata signer          |
    UnverifyNft,
}
#[allow(missing_docs)]
pub fn create_mint(
    accounts: create_mint::Accounts<Pubkey>,
    params: create_mint::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::CreateMint as u8, params)
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
#[allow(missing_docs)]
pub fn create_collection(
    accounts: create_collection::Accounts<Pubkey>,
    params: create_collection::Params,
) -> Instruction {
    accounts.get_instruction(
        crate::ID,
        ProgramInstruction::CreateCollection as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn edit_data(accounts: edit_data::Accounts<Pubkey>, params: edit_data::Params) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::EditData as u8, params)
}

#[allow(missing_docs)]
pub fn unverify_nft(
    accounts: unverify_nft::Accounts<Pubkey>,
    params: unverify_nft::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::UnverifyNft as u8, params)
}
