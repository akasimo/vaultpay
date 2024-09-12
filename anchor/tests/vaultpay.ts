import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vaultpay } from "../target/types/vaultpay";
import { MockYieldSource } from "../target/types/mock_yield_source";
import { Kamino } from "../target/types/kamino";
import { createAccount, createMint, mintTo } from "@solana/spl-token";
import { PublicKey, Keypair, SystemProgram, Commitment } from "@solana/web3.js"

describe("vaultpay", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const [initializer, user1, user2] = [new Keypair(), new Keypair(), new Keypair()];

  const vaultpayProgram = anchor.workspace.Vaultpay as Program<Vaultpay>;
  const kaminoProgram = anchor.workspace.Kamino as Program<Kamino>;
  let lendingMarket: anchor.web3.PublicKey;
  let lendingMarketKeypair: anchor.web3.Keypair;
  let lendingMarketAuthority: anchor.web3.PublicKey;
  let pyUsdMint: anchor.web3.PublicKey;

  const LENDING_MARKET_AUTH_SEED = "lma";
  const LENDING_MARKET_PDA_SEED = "lending_market";
  const LENDING_MARKET_QUOTE_CURRENCY = "PYUSD";

  let quoteCurrency = Buffer.alloc(32);

  it("Airdrop", async () => {
    await Promise.all([initializer, user1, user2].map(async (k) => {
      return await anchor.getProvider().connection.requestAirdrop(k.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL)
    })).then(confirmTxs);
  });

  it("Mint PYUSD", async () => {
    const mintAuthority = anchor.web3.Keypair.generate();
    pyUsdMint = await createMint(
      provider.connection,
      mintAuthority,
      mintAuthority.publicKey,
      null,
      6 // Assuming 6 decimals for pyUSD
    );

    const ata = await createAccount(anchor.getProvider().connection, initializer, pyUsdMint, initializer.publicKey);
    const signature = await mintTo(anchor.getProvider().connection, initializer, pyUsdMint, ata, initializer, 1e8 * 1e6);
    await confirmTx(signature);
  });

  it("Kamino lending market initialized!", async () => {
    // Add your test here.
    lendingMarketKeypair = Keypair.generate();
    lendingMarket = lendingMarketKeypair.publicKey;

    // Derive the lending market authority PDA
    lendingMarketAuthority = PublicKey.findProgramAddressSync(
      [Buffer.from(LENDING_MARKET_AUTH_SEED), lendingMarket.toBuffer()],
      kaminoProgram.programId
    )[0];

    Buffer.from(LENDING_MARKET_QUOTE_CURRENCY).copy(quoteCurrency);

    const quoteCurrencyArray = Array.from(quoteCurrency);

    await kaminoProgram.methods.initLendingMarket(
      quoteCurrencyArray // 32-byte array with "pyusd"
    )
    .accounts({
      lendingMarketOwner: provider.wallet.publicKey,
      lendingMarket: lendingMarket,
      lendingMarketAuthority: lendingMarketAuthority,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([lendingMarketKeypair]) // Add the new Keypair as a signer
    .rpc();
  });
});

const commitment: Commitment = "confirmed";

export const confirmTx = async (signature: string) => {
  const latestBlockhash = await anchor.getProvider().connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction(
      {
          signature,
          ...latestBlockhash,
      },
      commitment
  )
}

export const confirmTxs = async (signatures: string[]) => {
  await Promise.all(signatures.map(confirmTx))
}
