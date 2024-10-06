// tests/vaultpay.ts

import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Vaultpay } from "../target/types/vaultpay";
import { MockYieldSource } from "../target/types/mock_yield_source";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Commitment,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddress,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import {
  ASSOCIATED_PROGRAM_ID,
} from "@coral-xyz/anchor/dist/cjs/utils/token";
import { buildTxConfirmOrLog, confirmTx, confirmTxs, logBalances } from "./utils";

describe("vaultpay", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const vaultpayProgram = anchor.workspace.Vaultpay as Program<Vaultpay>;
  const mockYieldProgram = anchor.workspace.MockYieldSource as Program<MockYieldSource>;

  const authority = Keypair.generate();
  const user = Keypair.generate();
  const vendorAuthority = Keypair.generate();

  let tokenMint: PublicKey;
  let authorityTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let vendorTokenAccount: PublicKey;
  let treasuryTokenAccount: PublicKey;

  let yieldReservePDA: PublicKey;
  let yieldReserveBump: number;
  let reserveTokenAccount: PublicKey;

  let configPDA: PublicKey;
  let configBump: number;

  let vaultpayAuthorityPDA: PublicKey;
  let vaultpayAuthorityBump: number;

  let yieldAccountPDA: PublicKey;
  let yieldAccountBump: number;
  let yieldTokenAccount: PublicKey;

  let vendorPDA: PublicKey;
  let vendorBump: number;

  let subscriptionPDA: PublicKey;
  let subscriptionBump: number;

  const seed = new BN(12345); // Arbitrary seed value
  const platformFee = 500; // 5% fee (500 basis points)
  const minSubscriptionDuration = 30 * 24 * 60 * 60; // 30 days
  const maxSubscriptionDuration = 365 * 24 * 60 * 60; // 1 year


  it("Airdrop SOL to authority, user, and vendor", async () => {
    await Promise.all(
      [authority, user, vendorAuthority].map(async (k) => {
        const signature = await provider.connection.requestAirdrop(
          k.publicKey,
          2 * anchor.web3.LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(signature, "confirmed");
      })
    );
  });

  it("Create token mint and mint tokens to authority", async () => {
    tokenMint = await createMint(
      provider.connection,
      authority, // payer
      authority.publicKey, // mint authority
      null, // freeze authority
      6 // decimals
    );

    // Authority's token account
    const authorityAtaInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority, // payer
      tokenMint,
      authority.publicKey
    );
    authorityTokenAccount = authorityAtaInfo.address;

    // Mint tokens to authority's token account
    await mintTo(
      provider.connection,
      authority, // payer
      tokenMint,
      authorityTokenAccount,
      authority,
      1_000_000_000 // amount (e.g., 1000 tokens with 6 decimals)
    );

    // User's token account
    const userAtaInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user, // payer
      tokenMint,
      user.publicKey
    );
    userTokenAccount = userAtaInfo.address;

    // Vendor's token account
    const vendorAtaInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      vendorAuthority, // payer
      tokenMint,
      vendorAuthority.publicKey
    );
    vendorTokenAccount = vendorAtaInfo.address;
  });

  console.log("authority publicKey:", authority.publicKey.toString());
  console.log("user publicKey:", user.publicKey.toString());

  it("Initialize the mock yield reserve", async () => {
    // Derive the yield reserve PDA

    const printer = {
      authority: authority.publicKey,
      tokenMint,
      authorityTokenAccount,
      yieldReserve: yieldReservePDA,
      reserveTokenAccount: reserveTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    };

    console.log(printer);

    [yieldReservePDA, yieldReserveBump] =
      await PublicKey.findProgramAddress(
        [Buffer.from("yield_reserve"), tokenMint.toBuffer()],
        mockYieldProgram.programId
      );

    // Reserve token account (ATA of yield reserve)
    reserveTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      yieldReservePDA,
      true // allowOwnerOffCurve
    );

    const ix = await mockYieldProgram.methods
      .initialize(
        0.1, // APY of 10%
        new BN(500_000_000) // Initial deposit amount (e.g., 500 tokens)
      )
      .accountsPartial({
        authority: authority.publicKey,
        tokenMint,
        authorityTokenAccount,
        yieldReserve: yieldReservePDA,
        reserveTokenAccount: reserveTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }).instruction();
    
    const txSignature = await buildTxConfirmOrLog(
        authority,
        ix,
        mockYieldProgram,
        "init yield"
      )

    console.log("Mock Yield Reserve initialized:", txSignature);
  });

  it("Initialize the vaultpay program", async () => {
    // Derive the config PDA
    [configPDA, configBump] = await PublicKey.findProgramAddress(
      [Buffer.from("config"), tokenMint.toBuffer(), authority.publicKey.toBuffer()],
      vaultpayProgram.programId
    );

    // Treasury token account (ATA of config)
    treasuryTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      configPDA,
      true // allowOwnerOffCurve
    );

    const tx = await vaultpayProgram.methods
      .initialize(
        new BN(seed.toNumber()),
        platformFee,
        new BN(minSubscriptionDuration),
        new BN(maxSubscriptionDuration)
      )
      .accountsPartial({
        owner: authority.publicKey,
        supportedToken: tokenMint,
        treasury: treasuryTokenAccount,
        config: configPDA,
        yieldProgram: mockYieldProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    console.log("Vaultpay initialized:", tx);
  });

  it("Initialize user vault", async () => {
    // Derive the vaultpay authority PDA
    [vaultpayAuthorityPDA, vaultpayAuthorityBump] =
      await PublicKey.findProgramAddress(
        [Buffer.from("vaultpay_authority"), configPDA.toBuffer(), user.publicKey.toBuffer()],
        vaultpayProgram.programId
      );

    // Derive yield account PDA
    [yieldAccountPDA, yieldAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from("yield_account"), yieldReservePDA.toBuffer(), vaultpayAuthorityPDA.toBuffer()],
      mockYieldProgram.programId
    );

    // Transfer 1 SOL to vaultpay authority PDA
    const transferInstruction = SystemProgram.transfer({
      fromPubkey: user.publicKey,
      toPubkey: vaultpayAuthorityPDA,
      lamports: anchor.web3.LAMPORTS_PER_SOL, // 1 SOL
    });

    const transferTxSignature = await buildTxConfirmOrLog(
      user,
      transferInstruction,
      vaultpayProgram,
      "transfer 1 SOL to vaultpay authority PDA"
    );

    console.log("1 SOL transferred to vaultpay authority PDA:", transferTxSignature);

    // Verify the balance of vaultpay authority PDA
    const vaultpayAuthorityBalance = await provider.connection.getBalance(vaultpayAuthorityPDA);
    console.log("Vaultpay authority PDA balance:", vaultpayAuthorityBalance / anchor.web3.LAMPORTS_PER_SOL, "SOL");

    // Yield token account (ATA of yield account)
    yieldTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      yieldAccountPDA,
      true // allowOwnerOffCurve
    );

    console.log("Mock Yield Program ID:", mockYieldProgram.programId.toString());

    const printer = {
      user: user.publicKey,
      config: configPDA,
      tokenMint,
      yieldReserve: yieldReservePDA,
      vaultpayAuthority: vaultpayAuthorityPDA,
      yieldAccount: yieldAccountPDA,
      yieldTokenAccount: yieldTokenAccount,
      yieldProgram: mockYieldProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    };

    console.log(printer);

    const ix = await vaultpayProgram.methods
      .initUser()
      .accounts({
        user: user.publicKey,
        tokenMint,
        config: configPDA,
        yieldReserve: yieldReservePDA,
        vaultpayAuthority: vaultpayAuthorityPDA,
        yieldAccount: yieldAccountPDA,
        yieldTokenAccount: yieldTokenAccount,
        yieldProgram: mockYieldProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }).instruction();

    const txSignature = await buildTxConfirmOrLog(
      user,
      ix,
      vaultpayProgram,
      "init user"
    )

    console.log("User vault initialized:", txSignature);
  });

  xit("User deposits tokens into vault", async () => {
    // Mint tokens to user
    await mintTo(
      provider.connection,
      authority,
      tokenMint,
      userTokenAccount,
      authority,
      1_000_000_000 // Amount (e.g., 1000 tokens)
    );

    const depositAmount = new BN(500_000_000); // Deposit 500 tokens

    const tx = await vaultpayProgram.methods
      .deposit(depositAmount)
      .accountsPartial({
        user: user.publicKey,
        tokenMint,
        yieldReserve: yieldReservePDA,
        vaultpayAuthority: vaultpayAuthorityPDA,
        yieldAccount: yieldAccountPDA,
        userTokenAccount: userTokenAccount,
        yieldTokenAccount: yieldTokenAccount,
        reserveTokenAccount: reserveTokenAccount,
        yieldProgram: mockYieldProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("User deposited into vault:", tx);
  });

  xit("Initialize vendor", async () => {
    // Derive vendor PDA
    [vendorPDA, vendorBump] = await PublicKey.findProgramAddress(
      [Buffer.from("vendor"), configPDA.toBuffer(), vendorAuthority.publicKey.toBuffer()],
      vaultpayProgram.programId
    );

    const tx = await vaultpayProgram.methods
      .initVendor(new BN(12345), true) // seed and is_whitelisted
      .accountsPartial({
        vendorAuthority: vendorAuthority.publicKey,
        supportedToken: tokenMint,
        config: configPDA,
        vendor: vendorPDA,
        vendorTokenAccount: vendorTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([vendorAuthority])
      .rpc();

    console.log("Vendor initialized:", tx);
  });

  xit("Initialize subscription", async () => {
    // Derive subscription PDA
    [subscriptionPDA, subscriptionBump] = await PublicKey.findProgramAddress(
      [Buffer.from("subscription"), vendorPDA.toBuffer(), user.publicKey.toBuffer()],
      vaultpayProgram.programId
    );

    const amountPerPayment = new BN(100_000_000); // 100 tokens per payment
    const numberOfPayments = 3; // 3 payments
    const startTime = new BN(Math.floor(Date.now() / 1000)); // Now

    const tx = await vaultpayProgram.methods
      .initSubscription(
        new BN(67890), // seed
        amountPerPayment,
        numberOfPayments,
        startTime
      )
      .accountsPartial({
        user: user.publicKey,
        supportedToken: tokenMint,
        config: configPDA,
        vendor: vendorPDA,
        vendorAuthority: vendorAuthority.publicKey,
        subscription: subscriptionPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("Subscription initialized:", tx);
  });

  xit("Process payment", async () => {
    const tx = await vaultpayProgram.methods
      .processPayment()
      .accountsPartial({
        payer: user.publicKey,
        supportedToken: tokenMint,
        config: configPDA,
        subscription: subscriptionPDA,
        vendor: vendorPDA,
        vendorAuthority: vendorAuthority.publicKey,
        user: user.publicKey,
        vaultpayAuthority: vaultpayAuthorityPDA,
        yieldReserve: yieldReservePDA,
        yieldAccount: yieldAccountPDA,
        yieldTokenAccount: yieldTokenAccount,
        vendorTokenAccount: vendorTokenAccount,
        treasuryTokenAccount: treasuryTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("Payment processed:", tx);

    // Fetch subscription account and assert payments made increased
    const subscriptionAccount = await vaultpayProgram.account.subscription.fetch(subscriptionPDA);
    assert.equal(subscriptionAccount.paymentsMade, 1, "Payments made should be 1");
  });

  xit("Cancel subscription", async () => {
    const tx = await vaultpayProgram.methods
      .cancelSubscription()
      .accounts({
        user: user.publicKey,
        subscription: subscriptionPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("Subscription canceled:", tx);

    // Fetch subscription account and assert status is canceled
    const subscriptionAccount = await vaultpayProgram.account.subscription.fetch(subscriptionPDA);
    assert.equal(subscriptionAccount.status, 1, "Subscription status should be canceled (1)");
  });

  xit("Withdraw funds from vault", async () => {
    const withdrawAmount = new BN(100_000_000); // Withdraw 100 tokens

    const tx = await vaultpayProgram.methods
      .withdraw(withdrawAmount)
      .accountsPartial({
        user: user.publicKey,
        tokenMint,
        yieldReserve: yieldReservePDA,
        vaultpayAuthority: vaultpayAuthorityPDA,
        yieldAccount: yieldAccountPDA,
        userTokenAccount: userTokenAccount,
        yieldTokenAccount: yieldTokenAccount,
        reserveTokenAccount: reserveTokenAccount,
        yieldProgram: mockYieldProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("User withdrew from vault:", tx);
  });

  xit("Claim treasury funds", async () => {
    // Create authority's token account if not exists
    const authorityAtaInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority,
      tokenMint,
      authority.publicKey
    );
    authorityTokenAccount = authorityAtaInfo.address;

    const claimAmount = new BN(10_000_000); // Claim 10 tokens

    const tx = await vaultpayProgram.methods
      .claimTreasury(claimAmount)
      .accountsPartial({
        authority: authority.publicKey,
        supportedToken: tokenMint,
        config: configPDA,
        treasuryTokenAccount: treasuryTokenAccount,
        authorityTokenAccount: authorityTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    console.log("Treasury funds claimed:", tx);
  });
});
