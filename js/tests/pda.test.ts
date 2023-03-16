import {
  MasterEdition,
  Metadata,
} from "@metaplex-foundation/mpl-token-metadata";
import { test, expect } from "@jest/globals";
import { PublicKey, Keypair } from "@solana/web3.js";
import { getMasterEditionPda, getMetadataPda } from "../src/bindings";

test("Metaplex PDA", async () => {
  const mint = Keypair.generate().publicKey;
  const metadata = await Metadata.getPDA(mint);
  expect(metadata.toBase58()).toBe(getMetadataPda(mint).toBase58());
  const master = await MasterEdition.getPDA(mint);
  expect(master.toBase58()).toBe(getMasterEditionPda(mint).toBase58());
});
