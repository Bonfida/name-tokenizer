import { deserialize, Schema } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";

export const MINT_PREFIX = Buffer.from("tokenized_name");
export const COLLECTION_PREFIX = Buffer.from("collection");

export const METADATA_SIGNER = new PublicKey(
  "ARy9ZzW9qFCb8c8Lxi4NCph1TRNabUaMH5tj4e5pqwHb"
);

export enum Tag {
  Uninitialized = 0,
  CentralState = 1,
  ActiveRecord = 2,
  InactiveRecord = 3,
}

export class NftRecord {
  tag: Tag;
  nonce: number;
  nameAccount: PublicKey;
  owner: PublicKey;
  nftMint: PublicKey;

  static schema: Schema = new Map([
    [
      NftRecord,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["nonce", "u8"],
          ["nameAccount", [32]],
          ["owner", [32]],
          ["nftMint", [32]],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: number;
    nonce: number;
    nameAccount: Uint8Array;
    owner: Uint8Array;
    nftMint: Uint8Array;
  }) {
    this.tag = obj.tag as Tag;
    this.nonce = obj.nonce;
    this.nameAccount = new PublicKey(obj.nameAccount);
    this.owner = new PublicKey(obj.owner);
    this.nftMint = new PublicKey(obj.nftMint);
  }

  static deserialize(data: Buffer): NftRecord {
    return deserialize(this.schema, NftRecord, data);
  }

  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("NFT record not found");
    }
    return this.deserialize(accountInfo.data);
  }
  static async findKey(nameAccount: PublicKey, programId: PublicKey) {
    return await PublicKey.findProgramAddress(
      [Buffer.from("nft_record"), nameAccount.toBuffer()],
      programId
    );
  }
}
