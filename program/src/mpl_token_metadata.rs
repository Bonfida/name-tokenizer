// Adapted from mpl-token-metadata 5.1.1.

use std::{convert::TryFrom, io::ErrorKind};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
};

pub const ID: Pubkey = solana_program::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Uninitialized,
    EditionV1,
    MasterEditionV1,
    ReservationListV1,
    MetadataV1,
    ReservationListV2,
    MasterEditionV2,
    EditionMarker,
    UseAuthorityRecord,
    CollectionAuthorityRecord,
    TokenOwnedEscrow,
    TokenRecord,
    MetadataDelegate,
    EditionMarkerV2,
    HolderDelegate,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenStandard {
    NonFungible,
    FungibleAsset,
    Fungible,
    NonFungibleEdition,
    ProgrammableNonFungible,
    ProgrammableNonFungibleEdition,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    pub key: Key,
    pub update_authority: Pubkey,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<Creator>>,
    pub primary_sale_happened: bool,
    pub is_mutable: bool,
    pub edition_nonce: Option<u8>,
    pub token_standard: Option<TokenStandard>,
    pub collection: Option<Collection>,
    pub uses: Option<Uses>,
    pub collection_details: Option<CollectionDetails>,
    pub programmable_config: Option<ProgrammableConfig>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct Uses {
    pub use_method: UseMethod,
    pub remaining: u64,
    pub total: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub enum CollectionDetails {
    V1 { size: u64 },
    V2 { padding: [u8; 8] },
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub enum ProgrammableConfig {
    V1 { rule_set: Option<Pubkey> },
}

impl Metadata {
    pub const PREFIX: &'static [u8] = b"metadata";

    pub fn create_pda(
        mint: Pubkey,
        bump: u8,
    ) -> Result<Pubkey, solana_program::pubkey::PubkeyError> {
        Pubkey::create_program_address(&[Self::PREFIX, ID.as_ref(), mint.as_ref(), &[bump]], &ID)
    }

    pub fn find_pda(mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::PREFIX, ID.as_ref(), mint.as_ref()], &ID)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut data = data;
        Self::deserialize(&mut data)
    }

    pub fn safe_deserialize(data: &[u8]) -> Result<Self, std::io::Error> {
        if data.is_empty() || data[0] != Key::MetadataV1 as u8 {
            return Err(std::io::Error::new(ErrorKind::Other, "DataTypeMismatch"));
        }

        let mut data = data;
        let key = Key::deserialize(&mut data)?;
        let update_authority = Pubkey::deserialize(&mut data)?;
        let mint = Pubkey::deserialize(&mut data)?;
        let name = String::deserialize(&mut data)?;
        let symbol = String::deserialize(&mut data)?;
        let uri = String::deserialize(&mut data)?;
        let seller_fee_basis_points = u16::deserialize(&mut data)?;
        let creators = Option::<Vec<Creator>>::deserialize(&mut data)?;
        let primary_sale_happened = bool::deserialize(&mut data)?;
        let is_mutable = bool::deserialize(&mut data)?;
        let edition_nonce = Option::<u8>::deserialize(&mut data)?;

        let token_standard = Option::<TokenStandard>::deserialize(&mut data)
            .ok()
            .flatten();
        let collection = Option::<Collection>::deserialize(&mut data).ok().flatten();
        let uses = Option::<Uses>::deserialize(&mut data).ok().flatten();
        let collection_details = Option::<CollectionDetails>::deserialize(&mut data)
            .ok()
            .flatten();
        let programmable_config = Option::<ProgrammableConfig>::deserialize(&mut data)
            .ok()
            .flatten();

        Ok(Self {
            key,
            update_authority,
            mint,
            name,
            symbol,
            uri,
            seller_fee_basis_points,
            creators,
            primary_sale_happened,
            is_mutable,
            edition_nonce,
            token_standard,
            collection,
            uses,
            collection_details,
            programmable_config,
        })
    }
}

impl<'a> TryFrom<&AccountInfo<'a>> for Metadata {
    type Error = std::io::Error;

    fn try_from(account_info: &AccountInfo<'a>) -> Result<Self, Self::Error> {
        let mut data: &[u8] = &account_info.data.borrow();
        Self::deserialize(&mut data)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct MasterEdition {
    pub key: Key,
    pub supply: u64,
    pub max_supply: Option<u64>,
}

impl MasterEdition {
    pub const PREFIX: (&'static [u8], &'static [u8]) = (b"metadata", b"edition");

    pub fn create_pda(
        mint: Pubkey,
        bump: u8,
    ) -> Result<Pubkey, solana_program::pubkey::PubkeyError> {
        Pubkey::create_program_address(
            &[
                Self::PREFIX.0,
                ID.as_ref(),
                mint.as_ref(),
                Self::PREFIX.1,
                &[bump],
            ],
            &ID,
        )
    }

    pub fn find_pda(mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::PREFIX.0, ID.as_ref(), mint.as_ref(), Self::PREFIX.1],
            &ID,
        )
    }
}

fn invoke_cpi<'a>(
    accounts: Vec<AccountMeta>,
    data: Vec<u8>,
    account_infos: Vec<AccountInfo<'a>>,
    signer_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let instruction = Instruction {
        program_id: ID,
        accounts,
        data,
    };

    if signer_seeds.is_empty() {
        invoke(&instruction, &account_infos)
    } else {
        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}

pub struct CreateMetadataAccountV3CpiAccounts<'a, 'b> {
    pub metadata: &'b AccountInfo<'a>,
    pub mint: &'b AccountInfo<'a>,
    pub mint_authority: &'b AccountInfo<'a>,
    pub payer: &'b AccountInfo<'a>,
    pub update_authority: (&'b AccountInfo<'a>, bool),
    pub system_program: &'b AccountInfo<'a>,
    pub rent: Option<&'b AccountInfo<'a>>,
}

pub struct CreateMetadataAccountV3Cpi<'a, 'b> {
    program: &'b AccountInfo<'a>,
    accounts: CreateMetadataAccountV3CpiAccounts<'a, 'b>,
    args: CreateMetadataAccountV3InstructionArgs,
}

impl<'a, 'b> CreateMetadataAccountV3Cpi<'a, 'b> {
    pub fn new(
        program: &'b AccountInfo<'a>,
        accounts: CreateMetadataAccountV3CpiAccounts<'a, 'b>,
        args: CreateMetadataAccountV3InstructionArgs,
    ) -> Self {
        Self {
            program,
            accounts,
            args,
        }
    }

    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signer_seeds: &[&[&[u8]]]) -> ProgramResult {
        let a = &self.accounts;
        let mut metas = vec![
            AccountMeta::new(*a.metadata.key, false),
            AccountMeta::new_readonly(*a.mint.key, false),
            AccountMeta::new_readonly(*a.mint_authority.key, true),
            AccountMeta::new(*a.payer.key, true),
            AccountMeta::new_readonly(*a.update_authority.0.key, a.update_authority.1),
            AccountMeta::new_readonly(*a.system_program.key, false),
        ];
        let mut infos = vec![
            self.program.clone(),
            a.metadata.clone(),
            a.mint.clone(),
            a.mint_authority.clone(),
            a.payer.clone(),
            a.update_authority.0.clone(),
            a.system_program.clone(),
        ];
        if let Some(rent) = a.rent {
            metas.push(AccountMeta::new_readonly(*rent.key, false));
            infos.push(rent.clone());
        }
        invoke_cpi(metas, instruction_data(33, &self.args), infos, signer_seeds)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct CreateMetadataAccountV3InstructionArgs {
    pub data: DataV2,
    pub is_mutable: bool,
    pub collection_details: Option<CollectionDetails>,
}

pub struct CreateMasterEditionV3CpiAccounts<'a, 'b> {
    pub edition: &'b AccountInfo<'a>,
    pub mint: &'b AccountInfo<'a>,
    pub update_authority: &'b AccountInfo<'a>,
    pub token_program: &'b AccountInfo<'a>,
    pub system_program: &'b AccountInfo<'a>,
    pub rent: Option<&'b AccountInfo<'a>>,
    pub mint_authority: &'b AccountInfo<'a>,
    pub metadata: &'b AccountInfo<'a>,
    pub payer: &'b AccountInfo<'a>,
}

pub struct CreateMasterEditionV3Cpi<'a, 'b> {
    program: &'b AccountInfo<'a>,
    accounts: CreateMasterEditionV3CpiAccounts<'a, 'b>,
    args: CreateMasterEditionV3InstructionArgs,
}

impl<'a, 'b> CreateMasterEditionV3Cpi<'a, 'b> {
    pub fn new(
        program: &'b AccountInfo<'a>,
        accounts: CreateMasterEditionV3CpiAccounts<'a, 'b>,
        args: CreateMasterEditionV3InstructionArgs,
    ) -> Self {
        Self {
            program,
            accounts,
            args,
        }
    }

    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signer_seeds: &[&[&[u8]]]) -> ProgramResult {
        let a = &self.accounts;
        let mut metas = vec![
            AccountMeta::new(*a.edition.key, false),
            AccountMeta::new(*a.mint.key, false),
            AccountMeta::new_readonly(*a.update_authority.key, true),
            AccountMeta::new_readonly(*a.mint_authority.key, true),
            AccountMeta::new(*a.payer.key, true),
            AccountMeta::new(*a.metadata.key, false),
            AccountMeta::new_readonly(*a.token_program.key, false),
            AccountMeta::new_readonly(*a.system_program.key, false),
        ];
        let mut infos = vec![
            self.program.clone(),
            a.edition.clone(),
            a.mint.clone(),
            a.update_authority.clone(),
            a.mint_authority.clone(),
            a.payer.clone(),
            a.metadata.clone(),
            a.token_program.clone(),
            a.system_program.clone(),
        ];
        if let Some(rent) = a.rent {
            metas.push(AccountMeta::new_readonly(*rent.key, false));
            infos.push(rent.clone());
        }
        invoke_cpi(metas, instruction_data(17, &self.args), infos, signer_seeds)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct CreateMasterEditionV3InstructionArgs {
    pub max_supply: Option<u64>,
}

pub struct SetAndVerifyCollectionCpiAccounts<'a, 'b> {
    pub metadata: &'b AccountInfo<'a>,
    pub update_authority: &'b AccountInfo<'a>,
    pub collection_authority: &'b AccountInfo<'a>,
    pub payer: &'b AccountInfo<'a>,
    pub collection_mint: &'b AccountInfo<'a>,
    pub collection: &'b AccountInfo<'a>,
    pub collection_master_edition_account: &'b AccountInfo<'a>,
    pub collection_authority_record: Option<&'b AccountInfo<'a>>,
}

pub struct SetAndVerifyCollectionCpi<'a, 'b> {
    program: &'b AccountInfo<'a>,
    accounts: SetAndVerifyCollectionCpiAccounts<'a, 'b>,
}

impl<'a, 'b> SetAndVerifyCollectionCpi<'a, 'b> {
    pub fn new(
        program: &'b AccountInfo<'a>,
        accounts: SetAndVerifyCollectionCpiAccounts<'a, 'b>,
    ) -> Self {
        Self { program, accounts }
    }

    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signer_seeds: &[&[&[u8]]]) -> ProgramResult {
        let a = &self.accounts;
        let mut metas = vec![
            AccountMeta::new(*a.metadata.key, false),
            AccountMeta::new(*a.collection_authority.key, true),
            AccountMeta::new(*a.payer.key, true),
            AccountMeta::new_readonly(*a.update_authority.key, false),
            AccountMeta::new_readonly(*a.collection_mint.key, false),
            AccountMeta::new_readonly(*a.collection.key, false),
            AccountMeta::new_readonly(*a.collection_master_edition_account.key, false),
        ];
        let mut infos = vec![
            self.program.clone(),
            a.metadata.clone(),
            a.collection_authority.clone(),
            a.payer.clone(),
            a.update_authority.clone(),
            a.collection_mint.clone(),
            a.collection.clone(),
            a.collection_master_edition_account.clone(),
        ];
        if let Some(record) = a.collection_authority_record {
            metas.push(AccountMeta::new_readonly(*record.key, false));
            infos.push(record.clone());
        }
        invoke_cpi(metas, vec![25], infos, signer_seeds)
    }
}

pub struct UnverifyCollectionCpiAccounts<'a, 'b> {
    pub metadata: &'b AccountInfo<'a>,
    pub collection_authority: &'b AccountInfo<'a>,
    pub collection_mint: &'b AccountInfo<'a>,
    pub collection: &'b AccountInfo<'a>,
    pub collection_master_edition_account: &'b AccountInfo<'a>,
    pub collection_authority_record: Option<&'b AccountInfo<'a>>,
}

pub struct UnverifyCollectionCpi<'a, 'b> {
    program: &'b AccountInfo<'a>,
    accounts: UnverifyCollectionCpiAccounts<'a, 'b>,
}

impl<'a, 'b> UnverifyCollectionCpi<'a, 'b> {
    pub fn new(
        program: &'b AccountInfo<'a>,
        accounts: UnverifyCollectionCpiAccounts<'a, 'b>,
    ) -> Self {
        Self { program, accounts }
    }

    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signer_seeds: &[&[&[u8]]]) -> ProgramResult {
        let a = &self.accounts;
        let mut metas = vec![
            AccountMeta::new(*a.metadata.key, false),
            AccountMeta::new(*a.collection_authority.key, true),
            AccountMeta::new_readonly(*a.collection_mint.key, false),
            AccountMeta::new_readonly(*a.collection.key, false),
            AccountMeta::new_readonly(*a.collection_master_edition_account.key, false),
        ];
        let mut infos = vec![
            self.program.clone(),
            a.metadata.clone(),
            a.collection_authority.clone(),
            a.collection_mint.clone(),
            a.collection.clone(),
            a.collection_master_edition_account.clone(),
        ];
        if let Some(record) = a.collection_authority_record {
            metas.push(AccountMeta::new_readonly(*record.key, false));
            infos.push(record.clone());
        }
        invoke_cpi(metas, vec![22], infos, signer_seeds)
    }
}

pub struct UpdateMetadataAccountV2CpiAccounts<'a, 'b> {
    pub metadata: &'b AccountInfo<'a>,
    pub update_authority: &'b AccountInfo<'a>,
}

pub struct UpdateMetadataAccountV2Cpi<'a, 'b> {
    program: &'b AccountInfo<'a>,
    accounts: UpdateMetadataAccountV2CpiAccounts<'a, 'b>,
    args: UpdateMetadataAccountV2InstructionArgs,
}

impl<'a, 'b> UpdateMetadataAccountV2Cpi<'a, 'b> {
    pub fn new(
        program: &'b AccountInfo<'a>,
        accounts: UpdateMetadataAccountV2CpiAccounts<'a, 'b>,
        args: UpdateMetadataAccountV2InstructionArgs,
    ) -> Self {
        Self {
            program,
            accounts,
            args,
        }
    }

    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signer_seeds: &[&[&[u8]]]) -> ProgramResult {
        let a = &self.accounts;
        let metas = vec![
            AccountMeta::new(*a.metadata.key, false),
            AccountMeta::new_readonly(*a.update_authority.key, true),
        ];
        let infos = vec![
            self.program.clone(),
            a.metadata.clone(),
            a.update_authority.clone(),
        ];
        invoke_cpi(metas, instruction_data(15, &self.args), infos, signer_seeds)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct UpdateMetadataAccountV2InstructionArgs {
    pub data: Option<DataV2>,
    pub new_update_authority: Option<Pubkey>,
    pub primary_sale_happened: Option<bool>,
    pub is_mutable: Option<bool>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct DataV2 {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<Creator>>,
    pub collection: Option<Collection>,
    pub uses: Option<Uses>,
}

fn instruction_data<T: BorshSerialize>(discriminator: u8, args: &T) -> Vec<u8> {
    let mut data = vec![discriminator];
    data.extend(borsh::to_vec(args).expect("metadata instruction arguments serialize"));
    data
}

pub mod accounts {
    pub use super::{MasterEdition, Metadata};
}

pub mod instructions {
    pub use super::{
        CreateMasterEditionV3Cpi, CreateMasterEditionV3CpiAccounts,
        CreateMasterEditionV3InstructionArgs, CreateMetadataAccountV3Cpi,
        CreateMetadataAccountV3CpiAccounts, CreateMetadataAccountV3InstructionArgs,
        SetAndVerifyCollectionCpi, SetAndVerifyCollectionCpiAccounts, UnverifyCollectionCpi,
        UnverifyCollectionCpiAccounts, UpdateMetadataAccountV2Cpi,
        UpdateMetadataAccountV2CpiAccounts, UpdateMetadataAccountV2InstructionArgs,
    };
}

pub mod types {
    pub use super::{Collection, CollectionDetails, Creator, DataV2, ProgrammableConfig};
}
