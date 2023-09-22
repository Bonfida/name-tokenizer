import { beforeAll, expect, jest, test } from "@jest/globals";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";

import {
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddressSync,
  createTransferInstruction,
  MintLayout,
} from "@solana/spl-token";
import { TokenMint, signAndSendInstructions } from "@bonfida/utils";
import {
  createNameRegistry,
  getNameAccountKey,
  getHashedName,
} from "@bonfida/spl-name-service";
import crypto from "crypto";
import {
  createMint,
  createNft,
  redeemNft,
  withdrawTokens,
  NAME_TOKENIZER_ID_DEVNET,
} from "../src/bindings";
import { Tag, MINT_PREFIX, NftRecord } from "../src/state";
import { Metadata } from "@metaplex-foundation/mpl-token-metadata";

// Global state initialized once in test startup and cleaned up at test
// teardown.
let connection: Connection;
let feePayer: Keypair;
let programId: PublicKey;

beforeAll(async () => {
  connection = new Connection(
    "https://explorer-api.devnet.solana.com/ ",
    "confirmed"
  );
  feePayer = Keypair.generate();
  const tx = await connection.requestAirdrop(
    feePayer.publicKey,
    LAMPORTS_PER_SOL
  );
  await connection.confirmTransaction(tx, "confirmed");
  console.log(`Fee payer airdropped tx ${tx}`);
  programId = NAME_TOKENIZER_ID_DEVNET;
});

jest.setTimeout(1_500_000);

/**
 * Test scenario
 *
 * Create mint
 * Create collection
 * Create NFT
 * Send funds to the tokenized domain (tokens + SOL)
 * Withdraw funds
 * Transfer NFT to new wallet
 * Sends funds to the tokenized domain (tokens + SOL)
 * Withdraw funds
 * Sends funds to the tokenized domain (tokens + SOL)
 * Redeem NFT
 * Withdraw funds
 * Create NFT again
 * Verify metadata
 */

test("End to end test", async () => {
  /**
   * Test variables
   */
  const decimals = Math.pow(10, 6);
  const token = await TokenMint.init(connection, feePayer);
  const alice = Keypair.generate();
  const bob = Keypair.generate();
  const uri =
    "https://cloudflare-ipfs.com/ipfs/QmcvZWy8eanJvc96iraVdwNXNyT2bQ8ZQsZhETEcbrZJcJ";
  const mintAmount = 20 * decimals;
  const [centralKey] = await PublicKey.findProgramAddress(
    [NAME_TOKENIZER_ID_DEVNET.toBuffer()],
    NAME_TOKENIZER_ID_DEVNET
  );

  // Expected balances
  const bobExpectedBalance = { sol: 0, token: 0 };
  const aliceExpectedBalance = { sol: 0, token: 0 };

  /**
   * Create token ATA for Alice and Bob
   */

  const aliceTokenAtaKey = getAssociatedTokenAddressSync(
    token.token,
    alice.publicKey
  );
  const bobTokenAtaKey = getAssociatedTokenAddressSync(
    token.token,
    bob.publicKey
  );
  let ix = [
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      aliceTokenAtaKey,
      alice.publicKey,
      token.token
    ),
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      bobTokenAtaKey,
      bob.publicKey,
      token.token
    ),
  ];
  let tx = await signAndSendInstructions(connection, [], feePayer, ix);

  /**
   * Airdrop Alice
   */
  tx = await connection.requestAirdrop(alice.publicKey, LAMPORTS_PER_SOL);
  await connection.confirmTransaction(tx, "confirmed");
  aliceExpectedBalance.sol += LAMPORTS_PER_SOL;

  /**
   * Create domain name
   */
  const size = 100 + 96;
  const lamports = await connection.getMinimumBalanceForRentExemption(size);
  const name = crypto.randomBytes(10).toString();
  const hashedName = await getHashedName(name);
  const nameKey = await getNameAccountKey(hashedName);
  ix = [
    await createNameRegistry(
      connection,
      name,
      size,
      feePayer.publicKey,
      alice.publicKey,
      lamports
    ),
  ];
  tx = await signAndSendInstructions(connection, [], feePayer, ix);
  console.log(`Create domain tx ${tx}`);

  /**
   * Create mint
   */
  const [mintKey] = await PublicKey.findProgramAddress(
    [MINT_PREFIX, nameKey.toBuffer()],
    programId
  );
  ix = await createMint(nameKey, feePayer.publicKey, programId);
  tx = await signAndSendInstructions(connection, [], feePayer, ix);

  console.log(`Create mint ${tx}`);

  /**
   * Create Collection
   */

  // ix = await createCollection(feePayer.publicKey, programId);
  // tx = await signAndSendInstructions(connection, [], feePayer, ix);

  // console.log(`Create collection ${tx}`);

  /**
   * Create ATAs for Alice and Bob
   */
  const aliceNftAtaKey = await getAssociatedTokenAddressSync(
    mintKey,
    alice.publicKey
  );
  const bobNftAtaKey = await getAssociatedTokenAddressSync(
    mintKey,
    bob.publicKey
  );

  ix = [
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      aliceNftAtaKey,
      alice.publicKey,
      mintKey
    ),
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      bobNftAtaKey,
      bob.publicKey,
      mintKey
    ),
  ];
  tx = await signAndSendInstructions(connection, [], feePayer, ix);

  console.log(`Create Alice and Bob ATAs`);

  /**
   * Verify state
   */
  let info = await connection.getAccountInfo(mintKey);
  let mintInfo = MintLayout.decode(info?.data!);
  expect(mintInfo.decimals).toBe(0);
  expect(mintInfo.freezeAuthority?.toBase58()).toBe(centralKey.toBase58());
  expect(mintInfo.isInitialized).toBe(true);
  expect(mintInfo.mintAuthority?.toBase58()).toBe(centralKey.toBase58());
  expect(Number(mintInfo.supply)).toBe(0);

  /**
   * Create NFT
   */
  ix = await createNft(
    name,
    uri,
    nameKey,
    alice.publicKey,
    feePayer.publicKey,
    programId
  );
  tx = await signAndSendInstructions(connection, [alice], feePayer, ix);

  console.log(`Create NFT tx ${tx}`);

  /**
   * Verify state
   */
  info = await connection.getAccountInfo(mintKey);
  mintInfo = MintLayout.decode(info?.data!);
  expect(Number(mintInfo.supply)).toBe(1);

  const [nftRecordKey, nftRecordNonce] = await NftRecord.findKey(
    nameKey,
    programId
  );
  let nftRecord = await NftRecord.retrieve(connection, nftRecordKey);
  expect(nftRecord.nameAccount.toBase58()).toBe(nameKey.toBase58());
  expect(nftRecord.nftMint.toBase58()).toBe(mintKey.toBase58());
  expect(nftRecord.nonce).toBe(nftRecordNonce);
  expect(nftRecord.owner.toBase58()).toBe(alice.publicKey.toBase58());
  expect(nftRecord.tag).toBe(Tag.ActiveRecord);

  let aliceNftAta = await connection.getTokenAccountBalance(aliceNftAtaKey);
  expect(aliceNftAta.value.uiAmount).toBe(1);

  /**
   * Send funds to the tokenized domain (tokens + SOL)
   */
  const nftRecordTokenAtaKey = getAssociatedTokenAddressSync(
    token.token,
    nftRecordKey,
    true
  );
  ix = [
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      nftRecordTokenAtaKey,
      nftRecordKey,
      token.token
    ),
  ];
  await signAndSendInstructions(connection, [], feePayer, ix);
  await token.mintInto(nftRecordTokenAtaKey, mintAmount);
  await connection.requestAirdrop(nftRecordKey, LAMPORTS_PER_SOL / 2);

  aliceExpectedBalance.sol += LAMPORTS_PER_SOL / 2;
  aliceExpectedBalance.token += mintAmount;

  /**
   * Withdraw funds
   */
  ix = await withdrawTokens(
    mintKey,
    token.token,
    alice.publicKey,
    nftRecordKey,
    programId
  );
  tx = await signAndSendInstructions(connection, [alice], feePayer, ix);
  console.log(`Alice withdrew tokens ${tx}`);

  /**
   * Verify state
   */
  let fetchedSolBalance = await connection.getBalance(alice.publicKey);
  let fetchedTokenBalance = await connection.getTokenAccountBalance(
    aliceTokenAtaKey
  );

  expect(aliceExpectedBalance.sol).toBe(fetchedSolBalance);
  expect(aliceExpectedBalance.token.toString()).toBe(
    fetchedTokenBalance.value.amount
  );

  /**
   * Transfer NFT to new wallet
   */
  ix = [
    createTransferInstruction(aliceNftAtaKey, bobNftAtaKey, alice.publicKey, 1),
  ];
  tx = await signAndSendInstructions(connection, [alice], feePayer, ix);
  console.log(`Transfer NFT from Alice to Bob`);

  /**
   * Send funds to the tokenized domain (tokens + SOL)
   */
  await token.mintInto(nftRecordTokenAtaKey, mintAmount);
  await connection.requestAirdrop(nftRecordKey, LAMPORTS_PER_SOL / 2);

  bobExpectedBalance.sol += LAMPORTS_PER_SOL / 2;
  bobExpectedBalance.token += mintAmount;

  /**
   * Withdraw funds
   */
  ix = await withdrawTokens(
    mintKey,
    token.token,
    bob.publicKey,
    nftRecordKey,
    programId
  );
  tx = await signAndSendInstructions(connection, [bob], feePayer, ix);
  console.log(`Bob withdrew tokens ${tx}`);

  /**
   * Verify state
   */
  fetchedSolBalance = await connection.getBalance(bob.publicKey);
  fetchedTokenBalance = await connection.getTokenAccountBalance(bobTokenAtaKey);

  expect(bobExpectedBalance.sol).toBe(fetchedSolBalance);
  expect(bobExpectedBalance.token.toString()).toBe(
    fetchedTokenBalance.value.amount
  );

  /**
   * Sends funds to the tokenized domain (tokens + SOL)
   */
  await token.mintInto(nftRecordTokenAtaKey, mintAmount);
  await connection.requestAirdrop(nftRecordKey, LAMPORTS_PER_SOL / 2);

  bobExpectedBalance.sol += LAMPORTS_PER_SOL / 2;
  bobExpectedBalance.token += mintAmount;

  /**
   * Redeem NFT
   */
  ix = await redeemNft(nameKey, bob.publicKey, programId);
  tx = await signAndSendInstructions(connection, [bob], feePayer, ix);
  console.log(`Bob redeemed NFT ${tx}`);

  /**
   * Verify state
   */
  info = await connection.getAccountInfo(mintKey);
  mintInfo = MintLayout.decode(info?.data!);
  expect(Number(mintInfo.supply)).toBe(0);

  nftRecord = await NftRecord.retrieve(connection, nftRecordKey);
  expect(nftRecord.nameAccount.toBase58()).toBe(nameKey.toBase58());
  expect(nftRecord.nftMint.toBase58()).toBe(mintKey.toBase58());
  expect(nftRecord.nonce).toBe(nftRecordNonce);
  expect(nftRecord.owner.toBase58()).toBe(bob.publicKey.toBase58());
  expect(nftRecord.tag).toBe(Tag.InactiveRecord);

  /**
   * Withdraw funds
   */
  ix = await withdrawTokens(
    mintKey,
    token.token,
    bob.publicKey,
    nftRecordKey,
    programId
  );
  tx = await signAndSendInstructions(connection, [bob], feePayer, ix);
  console.log(`Bob withdrew tokens ${tx}`);

  /**
   * Verify state
   */
  fetchedSolBalance = await connection.getBalance(bob.publicKey);
  fetchedTokenBalance = await connection.getTokenAccountBalance(bobTokenAtaKey);

  expect(bobExpectedBalance.sol).toBe(fetchedSolBalance);
  expect(bobExpectedBalance.token.toString()).toBe(
    fetchedTokenBalance.value.amount
  );

  /**
   * Create NFT again
   */
  ix = await createNft(
    name,
    uri,
    nameKey,
    bob.publicKey,
    feePayer.publicKey,
    programId
  );
  tx = await signAndSendInstructions(connection, [bob], feePayer, ix);

  /**
   * Verify state
   */

  info = await connection.getAccountInfo(mintKey);
  mintInfo = MintLayout.decode(info?.data!);
  expect(mintInfo.decimals).toBe(0);
  expect(mintInfo.freezeAuthority?.toBase58()).toBe(centralKey.toBase58());
  expect(mintInfo.isInitialized).toBe(true);
  expect(mintInfo.mintAuthority?.toBase58()).toBe(centralKey.toBase58());
  expect(Number(mintInfo.supply)).toBe(1);

  nftRecord = await NftRecord.retrieve(connection, nftRecordKey);
  expect(nftRecord.nameAccount.toBase58()).toBe(nameKey.toBase58());
  expect(nftRecord.nftMint.toBase58()).toBe(mintKey.toBase58());
  expect(nftRecord.nonce).toBe(nftRecordNonce);
  expect(nftRecord.owner.toBase58()).toBe(bob.publicKey.toBase58());
  expect(nftRecord.tag).toBe(Tag.ActiveRecord);

  /**
   * Verify metadata
   */
  const metadata = await Metadata.findByMint(connection, mintKey);

  expect(metadata.data.data.name).toBe(name);
  expect(metadata.data.data.sellerFeeBasisPoints).toBe(500);
  expect(metadata.data.data.symbol).toBe(".sol");
  expect(metadata.data.data.uri).toBe(uri);
  expect(metadata.data.isMutable).toBe(1);
  expect(metadata.data.mint).toBe(mintKey.toBase58());
  expect(metadata.data.updateAuthority).toBe(centralKey.toBase58());

  expect(JSON.stringify(metadata.data.data.creators)).toBe(
    `[{"address":"${centralKey.toBase58()}","verified":1,"share":0},{"address":"94xt1Eyc56YDU6MtV7KsG8xfeRqd7z272g14tBHztnUM","verified":0,"share":100}]`
  );
});
