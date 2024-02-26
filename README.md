<h1 align="center">Name tokenizer</h1>
<br />
<p align="center">
<img width="250" src="https://i.imgur.com/nn7LMNV.png"/>
</p>
<p align="center">
<a href="https://twitter.com/bonfida">
<img src="https://img.shields.io/twitter/url?label=Bonfida&style=social&url=https%3A%2F%2Ftwitter.com%2Fbonfida">
</a>
</p>
<br />

<p align="center">
<strong>
Tokenize domain name into Metaplex NFTs
</strong>
</p>

<div align="center">
<img src="https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white" />
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
</div>

<br />
<h2 align="center">Table of contents</h2>
<br />

1. [Program ID](#program-id)
2. [Introduction](#introduction)
3. [Security](#security)
4. [Reproducible build](#build)
5. [Collection](#collection)
6. [Mint](#mint)
7. [NFT](#nft)
8. [Tests](#tests)
   - Rust
   - JS

<br />
<a name="program-id"></a>
<h2 align="center">Program ID</h2>
<br />

Mainnet program ID `nftD3vbNkNqfj2Sd3HZwbpw4BxxKWr4AjGb9X38JeZk`

<br />
<a name="introduction"></a>
<h2 align="center">Introduction</h2>
<br />

This program allows people to tokenize their domain name in NFTs that follow the [Metaplex standard](https://github.com/metaplex-foundation/metaplex-program-library/tree/master/token-metadata) with a creation/redemption mechanism.

<br />
<a name="build"></a>
<h2 align="center">Reproducible build</h2>
<br />

A reproducible build script (`build.sh`) can be used to build the program using docker

<br />
<a name="security"></a>
<h2 align="center">Security</h2>
<br />

For security disclosures or to report a bug, please visit [ImmuneFi](https://immunefi.com/bounty/bonfida/) for more information on our bug bounty program.

<br />
<a name="collection"></a>
<h2 align="center">Collection</h2>
<br />

NFTs are all part of a verified collection `E5ZnBpH9DYcxRkumKdS4ayJ3Ftb6o3E8wSbXw4N92GWg`.

<br />
<a name="mint"></a>
<h2 align="center">Mint</h2>
<br />

NFT mints are PDAs derived from the domain name key they represent. The derivation is made as follow:

```rust
pub const MINT_PREFIX: &[u8; 14] = b"tokenized_name";

// ...

let (mint, mint_nonce) = Pubkey::find_program_address(
    &[MINT_PREFIX, &accounts.name_account.key.to_bytes()],
    program_id,
);
```

<br />
<a name="nft"></a>
<h2 align="center">NFT</h2>
<br />

When a domain name is tokenized its ownership is transfered to a PDA that will be holding the domain while it's tokenized. In exchange, the program mints an NFT for the user. When redeeming the domain is transfered back to the NFT holder and the NFT burned.

During the tokenization process an `NftRecord` is created with the following state:

```rust
pub struct NftRecord {
    /// Tag
    pub tag: Tag,

    /// Nonce
    pub nonce: u8,

    /// Name account of the record
    pub name_account: Pubkey,

    /// Record owner
    pub owner: Pubkey,

    /// NFT mint
    pub nft_mint: Pubkey,
}
```

If funds are sent by mistake to the `NftRecord` instead of the NFT holder while the domain is tokenized the owner has the possibility to withdraw them. The "correct owner" is determined as follow:

- If the `NftRecord` is active i.e domain is tokenized: The correct owner is the NFT holder
- If `NftRecord` is inactive i.e the NFT has been redeemed: The correct owner is the last person who redeemed (`owner` field in the `NftRecord`)

<br />
<a name="tests"></a>
<h2 align="center">Tests</h2>
<br />

### Rust

Functional Rust tests can be run with

```
cargo test-bpf --features devnet
```

### JS

End to end tests can be run with

```
yarn jest
```
