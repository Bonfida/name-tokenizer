// This file is auto-generated. DO NOT EDIT
import BN from "bn.js";
import { Schema, serialize } from "borsh";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

export interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}
export class withdrawTokensInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      withdrawTokensInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 3;
  }
  serialize(): Uint8Array {
    return serialize(withdrawTokensInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    nft: PublicKey,
    nftOwner: PublicKey,
    nftRecord: PublicKey,
    tokenDestination: PublicKey,
    tokenSource: PublicKey,
    splTokenProgram: PublicKey,
    systemProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: nft,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nftOwner,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: nftRecord,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: tokenDestination,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: tokenSource,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class createNftInstruction {
  tag: number;
  name: string;
  uri: string;
  static schema: Schema = new Map([
    [
      createNftInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["name", "string"],
          ["uri", "string"],
        ],
      },
    ],
  ]);
  constructor(obj: { name: string; uri: string }) {
    this.tag = 1;
    this.name = obj.name;
    this.uri = obj.uri;
  }
  serialize(): Uint8Array {
    return serialize(createNftInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    mint: PublicKey,
    nftDestination: PublicKey,
    nameAccount: PublicKey,
    nftRecord: PublicKey,
    nameOwner: PublicKey,
    metadataAccount: PublicKey,
    centralState: PublicKey,
    feePayer: PublicKey,
    splTokenProgram: PublicKey,
    metadataProgram: PublicKey,
    systemProgram: PublicKey,
    splNameServiceProgram: PublicKey,
    rentAccount: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nftDestination,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nameAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nftRecord,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nameOwner,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: metadataAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: feePayer,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: metadataProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splNameServiceProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: rentAccount,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class createMintInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      createMintInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 0;
  }
  serialize(): Uint8Array {
    return serialize(createMintInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    mint: PublicKey,
    nameAccount: PublicKey,
    centralState: PublicKey,
    splTokenProgram: PublicKey,
    systemProgram: PublicKey,
    rentAccount: PublicKey,
    feePayer: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nameAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: rentAccount,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: feePayer,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class redeemNftInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      redeemNftInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 2;
  }
  serialize(): Uint8Array {
    return serialize(redeemNftInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    mint: PublicKey,
    nftSource: PublicKey,
    nftOwner: PublicKey,
    nftRecord: PublicKey,
    nameAccount: PublicKey,
    splTokenProgram: PublicKey,
    splNameServiceProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nftSource,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nftOwner,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: nftRecord,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: nameAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splNameServiceProgram,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
