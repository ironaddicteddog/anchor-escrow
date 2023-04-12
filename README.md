# Anchor Example: Escrow Program

> See this [doc](https://solmeet.gen3.network/notes/intro-to-anchor) for more implementation details

## Overview

Since this program is extended from the original [Escrow Program](https://github.com/paul-schaaf/solana-escrow), I assumed you have gone through the [original blog post](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/#instruction-rs-part-1-general-code-structure-and-the-beginning-of-the-escrow-program-flow) at least once.

However, there is one major difference between this exmaple and the original Escrow program: Instead of letting initializer create a token account to be reset to a PDA authority, we create `Vault Authority(PDA)`'s associated token account (ATA) directly without transferring authority.

#### Initialize

![](https://hackmd.io/_uploads/Hkn1gdtuj.png)

`Initializer` can send a transaction to the escrow program to initialize the Vault. In this transaction, two new accounts: `Vault Authority's ATA` and `Escrow State`, will be created and tokens (Token A) to be exchanged will be transferred from `Initializer` to `Vault`(short for vault authority's ATA).

#### Cancel

![](https://hackmd.io/_uploads/ry0GNdKdo.png)

`Initializer` can also send a transaction to the escrow program to cancel the demand of escrow. The tokens will be transferred back to the `Initializer` and both `Vault` and `Escrow State` will be closed in this case.

#### Exchange

![](https://hackmd.io/_uploads/HkhNE_tdi.png)

`Taker` can send a transaction to the escrow to exchange Token B for Token A. First, tokens (Token B) will be transferred from `Taker` to `Initializer`. Afterward, the tokens (Token A) kept in the Vault will be transferred to `Taker`. Finally, both `Vault` and `Escrow State` will be closed.

## Install, Build, Deploy and Test

Let's run the test once to see what happens.

### Install `anchor`

First, make sure that `anchor` is installed:

Install `avm`:

```bash
$ cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
...
```

Install latest `anchor` version:

```bash
$ avm install 0.27.0
...
$ avm use 0.27.0
...
```

> If you haven't installed `cargo`, please refer to this [doc](https://book.solmeet.dev/notes/solana-starter-kit#install-rust-and-solana-cli) for installation steps.

#### Extra Dependencies on Linux (Optional)

You may have to install some extra dependencies on Linux (ex. Ubuntu):

```bash
$ sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev
...
```

#### Verify the Installation

Check if Anchor is successfully installed:

```bash
$ anchor --version
anchor-cli 0.27.0
```

### Install Dependencies

Next, install dependencies:

```
$ yarn
```

### Build `anchor-escrow`

#### Update `program_id`

Get the public key of the deploy key. This keypair is generated automatically so a different key is exptected:

```bash
$ anchor keys list
anchor_escrow: Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS
```

Replace the default value of `program_id` with this new value:

```toml
# Anchor.toml

[programs.localnet]
anchor_escrow = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"

...
```

```rust
// lib.rs

...

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

...
```

Build the program:

```
$ anchor build
```

### Deploy `anchor-escrow`

Let's deploy the program. Notice that `anchor-escrow` will be deployed on a [mainnet-fork](https://github.com/DappioWonderland/solana) test validator run by Dappio:

```
$ solana config set --url localhost
...
```

```
$ anchor deploy
...

Program Id: Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS

Deploy success
```

Finally, run the test:

```
$ anchor test --skip-deploy --skip-build --skip-local-validator
```
