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
import { confirmTx, confirmTxs, logBalances, newMintToAta } from "./utils";
import { randomBytes } from "crypto"

describe("mock_yield_source", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.MockYieldSource as Program<MockYieldSource>;
    const provider = anchor.getProvider();

    const authority = Keypair.generate();
    const user = Keypair.generate();

    const seed = new BN(randomBytes(8));

    let tokenMint: PublicKey;
    let authorityTokenAccount: PublicKey;
    let userTokenAccount: PublicKey;
    let reserveTokenAccount: PublicKey;
    
    let vaultAuthority: PublicKey;
    // vaultAuthority = user.publicKey;
    let vaultReserveBump: number;

    let yieldReservePDA: PublicKey;
    let yieldReserveBump: number;

    let yieldAccountPDA: PublicKey;
    let yieldAccountBump: number;

    let yieldTokenAccount: PublicKey;

    it("Airdrop SOL to authority and user", async () => {
        await Promise.all([authority, user].map(async (k) => {
            return await anchor.getProvider().connection.requestAirdrop(k.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL)
        })).then(confirmTxs);
        [vaultAuthority, vaultReserveBump] = await PublicKey.findProgramAddressSync(
            [Buffer.from("vaultpay_authority"), user.publicKey.toBuffer()],
            program.programId
        );
        
    });

    it("Create token mint and mint tokens to authority", async () => {
        console.log("Creating token mint");
        tokenMint = await createMint(
            provider.connection,
            authority, // payer
            authority.publicKey, // mint authority
            null, // freeze authority
            6 // decimals
        );
        console.log("Token mint created");

        // Authority's token account
        const authorityAtaInfo = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            authority, // payer
            tokenMint,
            authority.publicKey
        );
        authorityTokenAccount = authorityAtaInfo.address;

        console.log("Authority's token account created");
        console.log("tokenMint:", tokenMint.toString());
        console.log("authorityTokenAccount:", authorityTokenAccount);
        console.log("authority publicKey:", authority.publicKey.toString());
        
        // Mint tokens to authority's token account
        await mintTo(
            provider.connection, //  connection
            authority, // payer
            tokenMint, // mint
            authorityTokenAccount, // destination
            authority, // authority
            1e6 * 1e6
        );
        console.log("Minted tokens to authority's token account");
        // User's token account
        const userAtaInfo = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            user, // payer
            tokenMint,
            user.publicKey
        );

        userTokenAccount = userAtaInfo.address;

        await mintTo(
            provider.connection,
            authority, // payer
            tokenMint,
            userTokenAccount,
            authority,
            1e3 * 1e6
        );
    });

    it("Initialize the yield reserve", async () => {
        // Derive the yield reserve PDA
        [yieldReservePDA, yieldReserveBump] = await PublicKey.findProgramAddressSync(
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
                100000, // APY of 10%
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
            })
            .signers([authority])
            .rpc();

        console.log("Initialize transaction signature:", tx);
    });

    it("Open a vault for the user", async () => {
        // Derive yield account PDA
        [yieldAccountPDA, yieldAccountBump] = await PublicKey.findProgramAddressSync(
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
            .accountsPartial({
                user: user.publicKey,
                tokenMint: tokenMint,
                vaultAuthority: vaultAuthority,
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

    const depositedAmount = new BN(1e3 * 1e6);

    it("User deposits tokens into the vault", async () => {
        // Mint tokens to user for deposit

        const balance = await logBalances(user.publicKey, "before deposit", tokenMint);

        const ix = await program.methods
            .deposit(depositedAmount)
            .accountsPartial({
                user: user.publicKey,
                tokenMint: tokenMint,
                vaultAuthority: vaultAuthority,
                userTokenAccount: userTokenAccount,
                yieldReserve: yieldReservePDA,
                yieldAccount: yieldAccountPDA,
                yieldTokenAccount: yieldTokenAccount,
                reserveTokenAccount: reserveTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .instruction();
        const tx = new anchor.web3.Transaction().add(ix);
        const txSignature = await program.provider.connection.sendTransaction(
            tx,
            [user],
            { skipPreflight: true }
        );
        console.log("Deposit transaction signature:", txSignature);
        await new Promise(resolve => setTimeout(resolve, 1000));
        const txDetails = await program.provider.connection.getTransaction(txSignature, {
          maxSupportedTransactionVersion: 0,
          commitment: "confirmed"
        });
        
        if (txDetails?.meta?.err) {
            console.log(txDetails);
            const logs = txDetails?.meta?.logMessages || null;

            if (logs) {
            console.log(logs);
            }
          throw new Error(`Transaction failed: ${JSON.stringify(txDetails.meta.err)}`);
        }
    });

    it("User withdraws tokens from the vault", async () => {

        const balance = await logBalances(user.publicKey, "before withdrawal", tokenMint);

        const ix = await program.methods
            .withdraw(depositedAmount)
            .accountsPartial({
                user: user.publicKey,
                tokenMint: tokenMint,
                vaultAuthority: vaultAuthority,
                userTokenAccount: userTokenAccount,
                yieldReserve: yieldReservePDA,
                yieldAccount: yieldAccountPDA,
                yieldTokenAccount: yieldTokenAccount,
                reserveTokenAccount: reserveTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .instruction();
        const tx = new anchor.web3.Transaction().add(ix);
        const txSignature = await program.provider.connection.sendTransaction(
            tx,
            [user],
            { skipPreflight: true }
        );
        console.log("Withdraw transaction signature:", txSignature);
        await new Promise(resolve => setTimeout(resolve, 1000));
        const txDetails = await program.provider.connection.getTransaction(txSignature, {
          maxSupportedTransactionVersion: 0,
          commitment: "confirmed"
        });
        
        if (txDetails?.meta?.err) {
            console.log(txDetails);
            const logs = txDetails?.meta?.logMessages || null;

            if (logs) {
            console.log(logs);
            }
          throw new Error(`Transaction failed: ${JSON.stringify(txDetails.meta.err)}`);
        }

        const balanceAfter = await logBalances(user.publicKey, "after withdrawal", tokenMint);
        
        // Assert that the balance has increased by the withdrawal amount
        // assert.equal(
        //     balanceAfter.balance.sub(balance.balance).toString(),
        //     depositedAmount.toString(),
        //     "User balance should increase by the withdrawn amount"
        // );
    });
});

// Helper function to airdrop SOL
async function airdropSol(publicKey: PublicKey, amountSol: number) {
    const provider = anchor.getProvider();
    const signature = await provider.connection.requestAirdrop(publicKey, amountSol * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature, "confirmed");
}
