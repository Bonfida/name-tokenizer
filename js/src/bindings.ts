import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import {
  withdrawTokensInstruction,
  createNftInstruction,
  createMintInstruction,
  redeemNftInstruction,
  createCollectionInstruction,
} from "./raw_instructions";
import { COLLECTION_PREFIX, MINT_PREFIX, NftRecord } from "./state";
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

export const NAME_TOKENIZER_ID = PublicKey.default;

export const NAME_TOKENIZER_ID_DEVNET = new PublicKey(
  "45gRSRZmK6NDEJrCZ72MMddjA1ozufq9YQpm41poPXCE"
);

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
    SYSVAR_RENT_PUBKEY
  );

  return [ix];
};

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
