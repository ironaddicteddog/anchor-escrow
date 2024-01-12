import * as anchor from "@coral-xyz/anchor";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { IDL } from "../target/types/anchor_escrow";
import { PublicKey, SystemProgram, TransactionMessage, VersionedTransaction } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("anchor-escrow", () => {
  const wallet = anchor.AnchorProvider.env().wallet;
  // Work-around: Get wallet private key
  const walletKey = NodeWallet.local().payer;

  anchor.setProvider(anchor.AnchorProvider.env());

  // CAUTION: if you are intended to use the program that is deployed by yourself,
  // please make sure that the programIDs are consistent
  const programId = new PublicKey("D3B39yETxmG29V3QPPNsdEzxQyonU5tjiwEr3svnMoRt");
  const program = new anchor.Program(IDL, programId, anchor.getProvider());

  let mintA = null as PublicKey;
  let mintB = null as PublicKey;
  let initializerTokenAccountA = null as PublicKey;
  let initializerTokenAccountB = null as PublicKey;
  let takerTokenAccountA = null as PublicKey;
  let takerTokenAccountB = null as PublicKey;

  const takerAmount = 1000;
  const initializerAmount = 500;

  // Main Roles
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const initializer = anchor.web3.Keypair.generate();
  const taker = anchor.web3.Keypair.generate();

  // Determined Seeds
  const stateSeed = "state";
  const authoritySeed = "authority";

  // Random Seed
  const randomSeed: anchor.BN = new anchor.BN(Math.floor(Math.random() * 100000000));

  // Derive PDAs: escrowStateKey, vaultKey, vaultAuthorityKey
  const escrowStateKey = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(stateSeed)), randomSeed.toArrayLike(Buffer, "le", 8)],
    program.programId
  )[0];

  const vaultAuthorityKey = PublicKey.findProgramAddressSync(
    [Buffer.from(authoritySeed, "utf-8")],
    program.programId
  )[0];
  let vaultKey = null as PublicKey;

  it("Initialize program state", async () => {
    // 1. Send 0.03 SOL from signer to payer.
    // Please make sure that there is at least 0.03 Devnet SOL in your signer wallet.
    // Claim Devnet SOL here: https://faucet.solana.com

    const latestBlockhash = await anchor.getProvider().connection.getLatestBlockhash();

    const airdropMessageV0 = new TransactionMessage({
      payerKey: wallet.publicKey,
      recentBlockhash: latestBlockhash.blockhash,
      instructions: [
        SystemProgram.transfer({
          fromPubkey: wallet.publicKey,
          toPubkey: payer.publicKey,
          lamports: 0.03 * 10 ** 9, // Convert SOL to lamports (1 SOL = 10^9 lamports)
        }),
      ],
    }).compileToV0Message();
    const airdropTx = new VersionedTransaction(airdropMessageV0);
    airdropTx.sign([walletKey]);

    await anchor.getProvider().connection.sendTransaction(airdropTx);

    // 2. Fund main roles: initializer and taker
    const fundingTxMessageV0 = new TransactionMessage({
      payerKey: payer.publicKey,
      recentBlockhash: latestBlockhash.blockhash,
      instructions: [
        SystemProgram.transfer({
          fromPubkey: payer.publicKey,
          toPubkey: initializer.publicKey,
          lamports: 10000000, // 0.01 SOL
        }),
        SystemProgram.transfer({
          fromPubkey: payer.publicKey,
          toPubkey: taker.publicKey,
          lamports: 10000000, // 0.01 SOL
        }),
      ],
    }).compileToV0Message();
    const fundingTx = new VersionedTransaction(fundingTxMessageV0);
    fundingTx.sign([payer]);

    // console.log(Buffer.from(fundingTx.serialize()).toString("base64"));
    const result = await anchor.getProvider().connection.sendRawTransaction(fundingTx.serialize());
    console.log(`https://solana.fm/tx/${result}?cluster=devnet-solana`);

    // 3. Create dummy token mints: mintA and mintB
    mintA = await createMint(anchor.getProvider().connection, payer, mintAuthority.publicKey, null, 0);
    mintB = await createMint(anchor.getProvider().connection, payer, mintAuthority.publicKey, null, 0);

    // 4. Create token accounts for dummy token mints and both main roles
    initializerTokenAccountA = await createAccount(
      anchor.getProvider().connection,
      initializer,
      mintA,
      initializer.publicKey
    );
    initializerTokenAccountB = await createAccount(
      anchor.getProvider().connection,
      initializer,
      mintB,
      initializer.publicKey
    );
    takerTokenAccountA = await createAccount(anchor.getProvider().connection, taker, mintA, taker.publicKey);
    takerTokenAccountB = await createAccount(anchor.getProvider().connection, taker, mintB, taker.publicKey);

    // 5. Mint dummy tokens to initializerTokenAccountA and takerTokenAccountB
    await mintTo(
      anchor.getProvider().connection,
      initializer,
      mintA,
      initializerTokenAccountA,
      mintAuthority,
      initializerAmount
    );
    await mintTo(anchor.getProvider().connection, taker, mintB, takerTokenAccountB, mintAuthority, takerAmount);

    const fetchedInitializerTokenAccountA = await getAccount(anchor.getProvider().connection, initializerTokenAccountA);
    const fetchedTakerTokenAccountB = await getAccount(anchor.getProvider().connection, takerTokenAccountB);

    assert.ok(Number(fetchedInitializerTokenAccountA.amount) == initializerAmount);
    assert.ok(Number(fetchedTakerTokenAccountB.amount) == takerAmount);
  });

  it("Initialize escrow", async () => {
    const _vaultKey = PublicKey.findProgramAddressSync(
      [vaultAuthorityKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mintA.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )[0];
    vaultKey = _vaultKey;

    const result = await program.methods
      .initialize(randomSeed, new anchor.BN(initializerAmount), new anchor.BN(takerAmount))
      .accounts({
        initializer: initializer.publicKey,
        vaultAuthority: vaultAuthorityKey,
        vault: vaultKey,
        mint: mintA,
        initializerDepositTokenAccount: initializerTokenAccountA,
        initializerReceiveTokenAccount: initializerTokenAccountB,
        escrowState: escrowStateKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([initializer])
      .rpc();
    console.log(`https://solana.fm/tx/${result}?cluster=devnet-solana`);

    let fetchedVault = await getAccount(anchor.getProvider().connection, vaultKey);
    let fetchedEscrowState = await program.account.escrowState.fetch(escrowStateKey);

    // Check that the new owner is the PDA.
    assert.ok(fetchedVault.owner.equals(vaultAuthorityKey));

    // Check that the values in the escrow account match what we expect.
    assert.ok(fetchedEscrowState.initializerKey.equals(initializer.publicKey));
    assert.ok(fetchedEscrowState.initializerAmount.toNumber() == initializerAmount);
    assert.ok(fetchedEscrowState.takerAmount.toNumber() == takerAmount);
    assert.ok(fetchedEscrowState.initializerDepositTokenAccount.equals(initializerTokenAccountA));
    assert.ok(fetchedEscrowState.initializerReceiveTokenAccount.equals(initializerTokenAccountB));
  });

  it("Exchange escrow state", async () => {
    const result = await program.methods
      .exchange()
      .accounts({
        taker: taker.publicKey,
        initializerDepositTokenMint: mintA,
        takerDepositTokenMint: mintB,
        takerDepositTokenAccount: takerTokenAccountB,
        takerReceiveTokenAccount: takerTokenAccountA,
        initializerDepositTokenAccount: initializerTokenAccountA,
        initializerReceiveTokenAccount: initializerTokenAccountB,
        initializer: initializer.publicKey,
        escrowState: escrowStateKey,
        vault: vaultKey,
        vaultAuthority: vaultAuthorityKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([taker])
      .rpc();
    console.log(`https://solana.fm/tx/${result}?cluster=devnet-solana`);

    let fetchedInitializerTokenAccountA = await getAccount(anchor.getProvider().connection, initializerTokenAccountA);
    let fetchedInitializerTokenAccountB = await getAccount(anchor.getProvider().connection, initializerTokenAccountB);
    let fetchedTakerTokenAccountA = await getAccount(anchor.getProvider().connection, takerTokenAccountA);
    let fetchedTakerTokenAccountB = await getAccount(anchor.getProvider().connection, takerTokenAccountB);

    assert.ok(Number(fetchedTakerTokenAccountA.amount) == initializerAmount);
    assert.ok(Number(fetchedInitializerTokenAccountA.amount) == 0);
    assert.ok(Number(fetchedInitializerTokenAccountB.amount) == takerAmount);
    assert.ok(Number(fetchedTakerTokenAccountB.amount) == 0);
  });

  it("Initialize escrow and cancel escrow", async () => {
    // Put back tokens into initializer token A account.
    await mintTo(
      anchor.getProvider().connection,
      initializer,
      mintA,
      initializerTokenAccountA,
      mintAuthority,
      initializerAmount
    );

    const initializedTx = await program.methods
      .initialize(randomSeed, new anchor.BN(initializerAmount), new anchor.BN(takerAmount))
      .accounts({
        initializer: initializer.publicKey,
        vaultAuthority: vaultAuthorityKey,
        vault: vaultKey,
        mint: mintA,
        initializerDepositTokenAccount: initializerTokenAccountA,
        initializerReceiveTokenAccount: initializerTokenAccountB,
        escrowState: escrowStateKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([initializer])
      .rpc();
    console.log(`https://solana.fm/tx/${initializedTx}?cluster=devnet-solana`);

    // Cancel the escrow.
    const canceledTX = await program.methods
      .cancel()
      .accounts({
        initializer: initializer.publicKey,
        mint: mintA,
        initializerDepositTokenAccount: initializerTokenAccountA,
        vault: vaultKey,
        vaultAuthority: vaultAuthorityKey,
        escrowState: escrowStateKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([initializer])
      .rpc();
    console.log(`https://solana.fm/tx/${canceledTX}?cluster=devnet-solana`);

    // Check the final owner should be the provider public key.
    const fetchedInitializerTokenAccountA = await getAccount(anchor.getProvider().connection, initializerTokenAccountA);

    assert.ok(fetchedInitializerTokenAccountA.owner.equals(initializer.publicKey));
    // Check all the funds are still there.
    assert.ok(Number(fetchedInitializerTokenAccountA.amount) == initializerAmount);
  });
});
