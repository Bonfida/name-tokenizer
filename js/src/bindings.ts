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
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Buffer } from "buffer";

const NAME_PROGRAM_ID = new PublicKey(
  "namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX"
);

const METADATA_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const PREFIX = "metadata";
const EDITION = "edition";

export const getMetadataPda = (mint: PublicKey) => {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(PREFIX), METADATA_ID.toBuffer(), mint.toBuffer()],
    METADATA_ID
  )[0];
};

export const getMasterEditionPda = (mint: PublicKey) => {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(PREFIX),
      METADATA_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from(EDITION),
    ],
    METADATA_ID
  )[0];
};

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
export const createMint = (
  nameAccount: PublicKey,
  feePayer: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = PublicKey.findProgramAddressSync(
    [programId.toBuffer()],
    programId
  );

  const [mint] = PublicKey.findProgramAddressSync(
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
export const createCollection = (feePayer: PublicKey, programId: PublicKey) => {
  const [centralKey] = PublicKey.findProgramAddressSync(
    [programId.toBuffer()],
    programId
  );
  const [collectionMint] = PublicKey.findProgramAddressSync(
    [COLLECTION_PREFIX, programId.toBuffer()],
    programId
  );
  const collectionMetadata = getMetadataPda(collectionMint);
  const editionAccount = getMasterEditionPda(collectionMint);
  const centralStateAta = getAssociatedTokenAddressSync(
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
    METADATA_ID,
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
export const createNft = (
  name: string,
  uri: string,
  nameAccount: PublicKey,
  nameOwner: PublicKey,
  feePayer: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = PublicKey.findProgramAddressSync(
    [programId.toBuffer()],
    programId
  );

  const [mint] = PublicKey.findProgramAddressSync(
    [MINT_PREFIX, nameAccount.toBuffer()],
    programId
  );

  const [nftRecord] = NftRecord.findKeySync(nameAccount, programId);
  const nftDestination = getAssociatedTokenAddressSync(mint, nameOwner);

  const metadataAccount = getMetadataPda(mint);

  const [collectionMint] = PublicKey.findProgramAddressSync(
    [COLLECTION_PREFIX, programId.toBuffer()],
    programId
  );
  const [collectionMetadata] = PublicKey.findProgramAddressSync(
    [Buffer.from(PREFIX), METADATA_ID.toBuffer(), collectionMint.toBuffer()],
    METADATA_ID
  );
  const editionAccount = getMasterEditionPda(collectionMint);

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
    METADATA_ID,
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
export const redeemNft = (
  nameAccount: PublicKey,
  nftOwner: PublicKey,
  programId: PublicKey
) => {
  const [mint] = PublicKey.findProgramAddressSync(
    [MINT_PREFIX, nameAccount.toBuffer()],
    programId
  );

  const [nftRecord] = NftRecord.findKeySync(nameAccount, programId);
  const nftSource = getAssociatedTokenAddressSync(mint, nftOwner);

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
export const withdrawTokens = (
  nftMint: PublicKey,
  tokenMint: PublicKey,
  nftOwner: PublicKey,
  nftRecord: PublicKey,
  programId: PublicKey
) => {
  const tokenDestination = getAssociatedTokenAddressSync(tokenMint, nftOwner);
  const tokenSource = getAssociatedTokenAddressSync(tokenMint, nftRecord, true);
  const nft = getAssociatedTokenAddressSync(nftMint, nftOwner);

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
