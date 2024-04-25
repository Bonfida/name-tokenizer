use {
    borsh::BorshDeserialize,
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
        pubkey::Pubkey,
    },
};

use crate::instruction::ProgramInstruction;

pub mod create_collection;
pub mod create_collection_core;
pub mod create_mint;
pub mod create_nft;
pub mod create_nft_core;
pub mod edit_data;
pub mod redeem_nft;
pub mod redeem_nft_core;
pub mod unverify_nft;
pub mod withdraw_tokens;
pub mod withdraw_tokens_core;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = FromPrimitive::from_u8(instruction_data[0])
            .ok_or(ProgramError::InvalidInstructionData)?;
        let instruction_data = &instruction_data[1..];
        msg!("Instruction unpacked");

        match instruction {
            ProgramInstruction::CreateMint => {
                msg!("Instruction: Create mint");
                create_mint::process(program_id, accounts)?;
            }
            ProgramInstruction::CreateCollection => {
                msg!("Instruction: Create collection");
                create_collection::process(program_id, accounts)?;
            }
            ProgramInstruction::CreateNft => {
                msg!("Instruction: Create NFT");
                let params = create_nft::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_nft::process(program_id, accounts, params)?;
            }
            ProgramInstruction::RedeemNft => {
                msg!("Instruction: Redeem NFT");
                redeem_nft::process(program_id, accounts)?;
            }
            ProgramInstruction::WithdrawTokens => {
                msg!("Instruction: Withdraw tokens");
                withdraw_tokens::process(program_id, accounts)?
            }
            ProgramInstruction::EditData => {
                msg!("Instruction: Edit data");
                let params = edit_data::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                edit_data::process(program_id, accounts, params)?
            }
            ProgramInstruction::UnverifyNft => {
                msg!("Instruction: Unverify NFT");
                let params = unverify_nft::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                unverify_nft::process(program_id, accounts, params)?
            }
            ProgramInstruction::CreateCollectionCore => {
                msg!("Instruction: Create Collection Core");
                create_collection_core::process(program_id, accounts)?
            }
            ProgramInstruction::CreateNftCore => {
                msg!("Instruction: Create NFT Core");
                let params = create_nft_core::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_nft_core::process(program_id, accounts, params)?;
            }
            ProgramInstruction::RedeemNftCore => {
                msg!("Instruction: Redeem NFT Core");
                redeem_nft_core::process(program_id, accounts)?;
            }
            ProgramInstruction::WithdrawTokensCore => {
                msg!("Instruction: Withdraw Tokens Core");
                withdraw_tokens_core::process(program_id, accounts)?;
            }
        }

        Ok(())
    }
}
