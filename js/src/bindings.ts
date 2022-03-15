import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import {
  withdrawTokensInstruction,
  createNftInstruction,
  createMintInstruction,
  redeemNftInstruction,
  createCollectionInstruction,
} from "./raw_instructions";
import {
  COLLECTION_PREFIX,
  MINT_PREFIX,
  NftRecord,
  METADATA_SIGNER,
} from "./state";
import {
  TOKEN_PROGRAM_ID,
  Token,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Metadata,
  MetadataProgram,
  MasterEdition,
} from "@metaplex-foundation/mpl-token-metadata";
import { NAME_PROGRAM_ID } from "@bonfida/spl-name-service";

/**
 * Mainnet program ID
 */
export const NAME_TOKENIZER_ID = new PublicKey(
  "nftD3vbNkNqfj2Sd3HZwbpw4BxxKWr4AjGb9X38JeZk"
);

/**
 * Devnet program ID (might not have the latest version deployed!)
 */
export const NAME_TOKENIZER_ID_DEVNET = new PublicKey(
  "45gRSRZmK6NDEJrCZ72MMddjA1ozufq9YQpm41poPXCE"
);

/**
 * This function can be used to create the mint of a domain name
 * @param nameAccount The domain name the mint represents
 * @param feePayer The fee payer of the transaction
 * @param programId The Name tokenizer program ID
 * @returns
 */
export const createMint = async (
  nameAccount: PublicKey,
  feePayer: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = await PublicKey.findProgramAddress(
    [programId.toBuffer()],
    programId
  );

  const [mint] = await PublicKey.findProgramAddress(
    [MINT_PREFIX, nameAccount.toBuffer()],
    programId
  );

  const ix = new createMintInstruction().getInstruction(
    programId,
    mint,
    nameAccount,
    centralKey,
    TOKEN_PROGRAM_ID,
    SystemProgram.programId,
    SYSVAR_RENT_PUBKEY,
    feePayer
  );

  return [ix];
};

/**
 * This function can be used to create the central state collection
 * @param feePayer The fee payer of the transaction
 * @param programId The Name tokenizer program ID
 * @returns
 */
export const createCollection = async (
  feePayer: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = await PublicKey.findProgramAddress(
    [programId.toBuffer()],
    programId
  );
  const [collectionMint] = await PublicKey.findProgramAddress(
    [COLLECTION_PREFIX, programId.toBuffer()],
    programId
  );
  const collectionMetadata = await Metadata.getPDA(collectionMint);
  const editionAccount = await MasterEdition.getPDA(collectionMint);

  const centralStateAta = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    collectionMint,
    centralKey,
    true
  );

  const ix = new createCollectionInstruction().getInstruction(
    programId,
    collectionMint,
    editionAccount,
    collectionMetadata,
    centralKey,
    centralStateAta,
    feePayer,
    TOKEN_PROGRAM_ID,
    MetadataProgram.PUBKEY,
    SystemProgram.programId,
    NAME_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    SYSVAR_RENT_PUBKEY
  );

  return [ix];
};

/**
 * This function can be used to create to wrap a domain name into an NFT
 * @param name The domain name (without .sol)
 * @param uri The URI of the metadata
 * @param nameAccount The domain name key
 * @param nameOwner The owner of the domain name to tokenize
 * @param feePayer The fee payer of the transaction
 * @param programId The Name tokenizer program ID
 * @returns
 */
export const createNft = async (
  name: string,
  uri: string,
  nameAccount: PublicKey,
  nameOwner: PublicKey,
  feePayer: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = await PublicKey.findProgramAddress(
    [programId.toBuffer()],
    programId
  );

  const [mint] = await PublicKey.findProgramAddress(
    [MINT_PREFIX, nameAccount.toBuffer()],
    programId
  );

  const [nftRecord] = await NftRecord.findKey(nameAccount, programId);

  const nftDestination = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    mint,
    nameOwner
  );

  const metadataAccount = await Metadata.getPDA(mint);

  const [collectionMint] = await PublicKey.findProgramAddress(
    [COLLECTION_PREFIX, programId.toBuffer()],
    programId
  );
  const collectionMetadata = await Metadata.getPDA(collectionMint);
  const editionAccount = await MasterEdition.getPDA(collectionMint);

  const ix = new createNftInstruction({ name, uri }).getInstruction(
    programId,
    mint,
    nftDestination,
    nameAccount,
    nftRecord,
    nameOwner,
    metadataAccount,
    editionAccount,
    collectionMetadata,
    collectionMint,
    centralKey,
    feePayer,
    TOKEN_PROGRAM_ID,
    MetadataProgram.PUBKEY,
    SystemProgram.programId,
    NAME_PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    METADATA_SIGNER
  );

  return [ix];
};

/**
 * This function can be used to unwrap a domain name that has been tokenized
 * @param nameAccount The domain name key
 * @param nftOwner The owner of the NFT to redeem
 * @param programId The Name tokenizer program ID
 * @returns
 */
export const redeemNft = async (
  nameAccount: PublicKey,
  nftOwner: PublicKey,
  programId: PublicKey
) => {
  const [mint] = await PublicKey.findProgramAddress(
    [MINT_PREFIX, nameAccount.toBuffer()],
    programId
  );

  const [nftRecord] = await NftRecord.findKey(nameAccount, programId);

  const nftSource = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    mint,
    nftOwner
  );

  const ix = new redeemNftInstruction().getInstruction(
    programId,
    mint,
    nftSource,
    nftOwner,
    nftRecord,
    nameAccount,
    TOKEN_PROGRAM_ID,
    NAME_PROGRAM_ID
  );

  return [ix];
};

/**
 * This function can be used to withdraw funds sent by mistake to an NftRecord while the domain was tokenized
 * @param nftMint The mint of the NFT
 * @param tokenMint The mint of the token to withdraw from the NftRecord
 * @param nftOwner The owner of the NFT (if the NFT has been redeemed it should be the latest person who redeemed)
 * @param nftRecord The NftRecord to which the funds were sent to
 * @param programId The Name tokenizer program ID
 * @returns
 */
export const withdrawTokens = async (
  nftMint: PublicKey,
  tokenMint: PublicKey,
  nftOwner: PublicKey,
  nftRecord: PublicKey,
  programId: PublicKey
) => {
  const tokenDestination = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    tokenMint,
    nftOwner
  );

  const tokenSource = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    tokenMint,
    nftRecord,
    true
  );

  const nft = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    nftMint,
    nftOwner
  );

  const ix = new withdrawTokensInstruction().getInstruction(
    programId,
    nft,
    nftOwner,
    nftRecord,
    tokenDestination,
    tokenSource,
    TOKEN_PROGRAM_ID,
    SystemProgram.programId
  );

  return [ix];
};
