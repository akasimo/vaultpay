import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { MockYieldSource } from "../target/types/mock_yield_source";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
    createMint,
    getOrCreateAssociatedTokenAccount,
    getAssociatedTokenAddress,
    mintTo,
    TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("mock_yield_source", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.MockYieldSource as Program<MockYieldSource>;
    const provider = anchor.getProvider();

    const authority = Keypair.generate();
    const user = Keypair.generate();

    let tokenMint: PublicKey;
    let authorityTokenAccount: PublicKey;
    let userTokenAccount: PublicKey;
    let reserveTokenAccount: PublicKey;

    let yieldReservePDA: PublicKey;
    let yieldReserveBump: number;

    let yieldAccountPDA: PublicKey;
    let yieldAccountBump: number;

    let yieldTokenAccount: PublicKey;

    it("Airdrop SOL to authority and user", async () => {
        await airdropSol(authority.publicKey, 2);
        await airdropSol(user.publicKey, 2);
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
        authorityTokenAccount = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            authority, // payer
            tokenMint,
            authority.publicKey
        );

        // Mint tokens to authority's token account
        await mintTo(
            provider.connection,
            authority, // payer
            tokenMint,
            authorityTokenAccount.address,
            authority,
            1_000_000_000 // amount (e.g., 1000 tokens with 6 decimals)
        );

        // User's token account
        userTokenAccount = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            user, // payer
            tokenMint,
            user.publicKey
        );
    });

    it("Initialize the yield reserve", async () => {
        // Derive the yield reserve PDA
        [yieldReservePDA, yieldReserveBump] = await PublicKey.findProgramAddress(
            [Buffer.from("yield_reserve"), tokenMint.toBuffer()],
            program.programId
        );

        // Reserve token account (ATA of yield reserve)
        reserveTokenAccount = await getAssociatedTokenAddress(
            tokenMint,
            yieldReservePDA,
            true // allowOwnerOffCurve
        );

        const tx = await program.methods
            .initialize(
                0.1, // APY of 10%
                new BN(500_000_000) // Initial deposit amount (e.g., 500 tokens)
            )
            .accounts({
                authority: authority.publicKey,
                tokenMint: tokenMint,
                authorityTokenAccount: authorityTokenAccount.address,
                yieldReserve: yieldReservePDA,
                reserveTokenAccount: reserveTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([authority])
            .rpc();

        console.log("Initialize transaction signature:", tx);
    });

    it("Open a vault for the user", async () => {
        // Derive yield account PDA
        [yieldAccountPDA, yieldAccountBump] = await PublicKey.findProgramAddress(
            [Buffer.from("yield_account"), yieldReservePDA.toBuffer(), user.publicKey.toBuffer()],
            program.programId
        );

        // Yield token account (ATA of yield account)
        yieldTokenAccount = await getAssociatedTokenAddress(
            tokenMint,
            yieldAccountPDA,
            true // allowOwnerOffCurve
        );

        const tx = await program.methods
            .openVault()
            .accounts({
                user: user.publicKey,
                tokenMint: tokenMint,
                yieldReserve: yieldReservePDA,
                yieldAccount: yieldAccountPDA,
                yieldTokenAccount: yieldTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();

        console.log("OpenVault transaction signature:", tx);
    });

    it("User deposits tokens into the vault", async () => {
        // Mint tokens to user for deposit
        await mintTo(
            provider.connection,
            authority,
            tokenMint,
            userTokenAccount.address,
            authority,
            500_000_000 // Amount (e.g., 500 tokens)
        );

        const depositAmount = new BN(100_000_000); // Deposit 100 tokens

        const tx = await program.methods
            .deposit(depositAmount)
            .accounts({
                user: user.publicKey,
                tokenMint: tokenMint,
                userTokenAccount: userTokenAccount.address,
                yieldReserve: yieldReservePDA,
                yieldAccount: yieldAccountPDA,
                yieldTokenAccount: yieldTokenAccount,
                reserveTokenAccount: reserveTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();

        console.log("Deposit transaction signature:", tx);
    });

    it("User withdraws tokens from the vault", async () => {
        const withdrawAmount = new BN(50_000_000); // Withdraw 50 tokens

        const tx = await program.methods
            .withdraw(withdrawAmount)
            .accounts({
                user: user.publicKey,
                tokenMint: tokenMint,
                userTokenAccount: userTokenAccount.address,
                yieldReserve: yieldReservePDA,
                yieldAccount: yieldAccountPDA,
                yieldTokenAccount: yieldTokenAccount,
                reserveTokenAccount: reserveTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();

        console.log("Withdraw transaction signature:", tx);
    });

    it("Check balances after withdrawal", async () => {
        // Fetch user token account balance
        const userTokenAccountInfo = await provider.connection.getTokenAccountBalance(userTokenAccount.address);
        console.log("User token account balance after withdrawal:", userTokenAccountInfo.value.uiAmount);

        // Fetch yield account data
        const yieldAccountInfo = await program.account.yieldAccount.fetch(yieldAccountPDA);
        console.log("Yield account deposited amount:", yieldAccountInfo.depositedAmount.toNumber());
        console.log("Yield account unclaimed yield:", yieldAccountInfo.unclaimedYield.toNumber());

        // Assertions (adjust numbers based on expected behavior)
        assert.equal(yieldAccountInfo.depositedAmount.toNumber(), 50_000_000, "Deposited amount should be 50 tokens");
        // Additional assertions can be added here
    });
});

// Helper function to airdrop SOL
async function airdropSol(publicKey: PublicKey, amountSol: number) {
    const provider = anchor.getProvider();
    const signature = await provider.connection.requestAirdrop(publicKey, amountSol * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature, "confirmed");
}
