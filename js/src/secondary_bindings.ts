import { Connection, PublicKey } from "@solana/web3.js";
import { NAME_TOKENIZER_ID } from "./bindings";

/**
 * This function can be used to retrieve the NFTs of an owner
 * @param connection A solana RPC connection
 * @param owner The owner to retrieve NFTs for
 * @returns
 */
export const getNftForOwner = async (
  connection: Connection,
  owner: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "3",
      },
    },
    {
      memcmp: {
        offset: 1 + 1 + 32,
        bytes: owner.toBase58(),
      },
    },
  ];

  const result = await connection.getProgramAccounts(NAME_TOKENIZER_ID, {
    filters,
  });

  return result;
};

/**
 * This function can used to retrieve the NFT record for a name account
 * @param connection A solana RPC connection
 * @param nameAccount The name account to retrieve the NftRecord for
 * @returns
 */
export const getMintFromNameAccount = async (
  connection: Connection,
  nameAccount: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "3",
      },
    },
    {
      memcmp: {
        offset: 1 + 1,
        bytes: nameAccount.toBase58(),
      },
    },
  ];

  const result = await connection.getProgramAccounts(NAME_TOKENIZER_ID, {
    filters,
  });

  return result;
};

/**
 * This function can be used to retrieve a NFT Record given a mint
 *
 * @param connection A solana RPC connection
 * @param mint The mint of the NFT Record
 * @returns
 */
export const getRecordFromMint = async (
  connection: Connection,
  mint: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "3",
      },
    },
    {
      memcmp: {
        offset: 1 + 1 + 32 + 32,
        bytes: mint.toBase58(),
      },
    },
  ];

  const result = await connection.getProgramAccounts(NAME_TOKENIZER_ID, {
    filters,
  });

  return result;
};

/**
 * This function can be used to retrieve all the active NFT record
 * @param connection A solana RPC connection
 * @returns
 */
export const getActiveRecords = async (connection: Connection) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "3",
      },
    },
  ];

  const result = await connection.getProgramAccounts(NAME_TOKENIZER_ID, {
    filters,
  });

  return result;
};
