import { Connection, PublicKey } from "@solana/web3.js";
import { NAME_TOKENIZER_ID } from "./bindings";

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
