# Anchor Example: Escrow Program

- See this [doc](https://hackmd.io/@ironaddicteddog/solana-anchor-escrow) for more implementation details

## Overview

Since this program is extended from the original [Escrow Program](https://github.com/paul-schaaf/solana-escrow), I assumed you have gone through the [original blog post](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/#instruction-rs-part-1-general-code-structure-and-the-beginning-of-the-escrow-program-flow) at least once.

However, there is one major difference between this exmaple and the original Escrow program: Instead of letting initializer create a token account to be reset to a PDA authority, we create a token account `Vault` that has both a PDA key and a PDA authority.

### Initialize

![](https://i.imgur.com/VmRKZUy.png)

`Initializer` can send a transaction to the escrow program to initialize the Vault. In this transaction, two new accounts: `Vault` and `EscrowAccount`, will be created and tokens (Token A) to be exchanged will be transfered from `Initializer` to `Vault`.

### Cancel

![](https://i.imgur.com/f6ahGXy.png)

`Initializer` can also send a transaction to the escrow program to cancel the demand of escrow. The tokens will be transfered back to the `Initialzer` and both `Vault` and `EscrowAccount` will be closed in this case.

### Exchange

![](https://i.imgur.com/MzG26dm.png)

`Taker` can send a transaction to the escrow to exchange Token B for Token A. First, tokens (Token B) will be transfered from `Taker` to `Initializer`. Afterward, the tokens (Token A) kept in the Vault will be transfered to `Taker`. Finally, both `Vault` and `EscrowAccount` will be closed.

## Build, Deploy and Test

Let's run the test once to see what happens.

First, install dependencies:

```
$ yarn
```

Next, we will build and deploy the program via Anchor.

Get the program ID:

```
$ anchor keys list
anchor_escrow: AGtT2X117M7Lx1PeXQrknorvwApEdBSUsAiYA2R2QESd
```

Here, make sure you update your program ID in `Anchor.toml` and `lib.rs`.

Build the program:

```
$ anchor build
```

Let's deploy the program. Notice that `anchor-escrow` will be deployed on a [mainnet-fork](https://github.com/DappioWonderland/solana) test validator run by Dappio:

```
$ anchor deploy
...

Program Id: AGtT2X117M7Lx1PeXQrknorvwApEdBSUsAiYA2R2QESd

Deploy success
```

Finally, run the test:

```
$ anchor test --skip-build --skip-deploy
```
