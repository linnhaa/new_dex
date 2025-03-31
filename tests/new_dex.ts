import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NewDex } from "../target/types/new_dex";
import {
  PublicKey,
  SystemProgram,
  Keypair,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("initialize-pool", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.NewDex as Program<NewDex>;
  const authority = (provider.wallet as anchor.Wallet).payer; // Authority = Wallet

  let poolPda: PublicKey;
  let solVaultPda: PublicKey;
  let tokenVault: PublicKey;
  let tokenMint: PublicKey;
  let userTokenAccount: PublicKey;
  let lp: Keypair;
  let lpInfoPda: PublicKey;

  //
  // ------------------------------------------------
  // Setup trước mỗi file test
  // ------------------------------------------------
  before(async () => {
    console.log("🔹 Generating new token mint...");
    
    // (1) Tạo token SPL mới
    const mint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      9 // 9 chữ số thập phân
    );
    tokenMint = mint;
    console.log("✅ New Token Mint:", tokenMint.toBase58());

    // (2) Tính PDA cho Pool
    [poolPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("POOL")],
      program.programId
    );

    [solVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("sol_vault"), poolPda.toBuffer()],
      program.programId
    );

    // (3) Tạo ATA (tokenVault) cho Pool
    const tokenVaultAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority, 
      tokenMint,
      poolPda,
      true // allowOwnerOffCurve = true (vì chủ sở hữu là PDA)
    );
    tokenVault = tokenVaultAccount.address;
    console.log("✅ Token Vault Address:", tokenVault.toBase58());

    // (4) Tạo LP (người cung cấp thanh khoản)
    lp = Keypair.generate();
    console.log("✅ New LP Generated:", lp.publicKey.toBase58());

    // (5) Airdrop SOL cho LP
    const airdropTx = await provider.connection.requestAirdrop(
      lp.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    const latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: airdropTx,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    }, "finalized");
    console.log("✅ LP Airdropped 2 SOL");

    // (6) Tạo ATA cho LP
    const lpTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      lp,
      tokenMint,
      lp.publicKey
    );
    userTokenAccount = lpTokenAccount.address;
    console.log("✅ LP Token Account:", userTokenAccount.toBase58());

    // (7) Mint 1000 token cho LP
    await mintTo(
      provider.connection,
      authority,
      tokenMint,
      userTokenAccount,
      authority,
      1000 * 10 ** 9
    );
    console.log("✅ Minted 1000 tokens to LP's account");

    // (8) PDA lp_info
    [lpInfoPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lp_info"), poolPda.toBuffer(), lp.publicKey.toBuffer()],
      program.programId
    );
    console.log("✅ LP Info PDA:", lpInfoPda.toBase58());
  });

  //
  // Hàm fetch LPInfo state
  //
  const fetchLpInfoState = async () => {
    try {
      const lpInfoState = await program.account.lpInfo.fetch(lpInfoPda);
      console.log("🔍 LP Info State:");
      console.log("   Owner:", lpInfoState.owner.toBase58());
      console.log("   SOL Amount:", lpInfoState.solAmount.toString());
      console.log("   Token Amount:", lpInfoState.tokenAmount.toString());
    } catch (error) {
      console.log("❌ LP Info not initialized yet!");
    }
  };

  //
  // Hàm fetch Pool state
  //
  const fetchPoolState = async () => {
    const poolState = await program.account.pool.fetch(poolPda);
    console.log("🔍 Pool State:");
    console.log("   Authority:", poolState.authority.toBase58());
    console.log("   Token Mint:", poolState.tokenMint.toBase58());
    console.log("   SOL Vault:", poolState.solVault.toBase58());
    console.log("   Token Vault:", poolState.tokenVault.toBase58());
    console.log("   SOL Amount:", poolState.solAmount.toString());
    console.log("   Token Amount:", poolState.tokenAmount.toString());
  };

  //
  // ------------------------------------------------
  // TEST #1: Initialize
  // ------------------------------------------------
  it("Initializes the pool", async () => {
    await program.methods
      .initialize()
      .accountsPartial({
        pool: poolPda,
        tokenMint: tokenMint,
        tokenVault: tokenVault,
        solVault: solVaultPda,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    console.log("✅ Pool initialized successfully!");
    
    // Kiểm tra
    await fetchPoolState();
  });

  //
  // ------------------------------------------------
  // TEST #2: Check LP info state (trước add)
  // ------------------------------------------------
  it("Check LP info state before adding liquidity", async () => {
    console.log("📌 LP info state before adding liquidity:");
    await fetchLpInfoState();
  });

  //
  // ------------------------------------------------
  // TEST #3: Add liquidity
  // ------------------------------------------------
  it("Adds liquidity to the pool", async () => {
    const solAmount = 1 * LAMPORTS_PER_SOL;
    const tokenAmount = 100 * 10 ** 9;

    await program.methods
      .addliquidity(new anchor.BN(solAmount), new anchor.BN(tokenAmount))
      .accountsPartial({
        pool: poolPda,
        lp: lp.publicKey,
        tokenMint: tokenMint,
        userTokenAccount: userTokenAccount,
        tokenVault: tokenVault,
        solVault: solVaultPda,
        lpInfo: lpInfoPda,
      })
      .signers([lp])
      .rpc();

    console.log("✅ Successfully added liquidity!");
    console.log("📌 Pool state after adding liquidity:");
    await fetchPoolState();
    console.log("📌 LP info state after adding liquidity:");
    await fetchLpInfoState();
  });

  //
  // ------------------------------------------------
  // TEST #4: Remove liquidity
  // ------------------------------------------------
  it("Removes liquidity from the pool", async () => {
    const solWithdrawAmount = 0.2 * LAMPORTS_PER_SOL;
    const tokenWithdrawAmount = 10 * 10 ** 9;

    console.log("📌 LP info state before removing liquidity:");
    await fetchLpInfoState();

    await program.methods
      .removeliquidity(
        new anchor.BN(solWithdrawAmount), 
        new anchor.BN(tokenWithdrawAmount)
      )
      .accountsPartial({
        pool: poolPda,
        lp: lp.publicKey,
        tokenMint: tokenMint,
        userTokenAccount: userTokenAccount,
        tokenVault: tokenVault,
        solVault: solVaultPda,
        lpInfo: lpInfoPda,
      })
      .signers([lp])
      .rpc();
    
    console.log("✅ Successfully removed liquidity!");
    console.log("📌 Pool state after removing liquidity:");
    await fetchPoolState();
    console.log("📌 LP info state after removing liquidity:");
    await fetchLpInfoState();
  });

  //
  // ------------------------------------------------
  // TEST #5: Swap some SOL for token
  // ------------------------------------------------
  it("Swaps some SOL for tokens", async () => {
    // Trader (ở đây dùng luôn lp) muốn swap 0.3 SOL
    const solAmount = 0.3 * LAMPORTS_PER_SOL;

    console.log("📌 Pool state before swap SOL->Token:");
    await fetchPoolState();

    await program.methods
      .swapsol(new anchor.BN(solAmount)) // Tên hàm: swapSol
      .accountsPartial({
        // Thay vì .accountsPartial(), ta dùng .accounts() cũng được
        pool: poolPda,
        trader: lp.publicKey,            // sign bởi lp
        tokenMint: tokenMint,
        traderTokenAccount: userTokenAccount,
        tokenVault: tokenVault,
        solVault: solVaultPda,
      })
      .signers([lp])
      .rpc();

    console.log(`✅ Swapped ${solAmount} lamports worth of SOL for tokens!`);
    console.log("📌 Pool state after swap SOL->Token:");
    await fetchPoolState();
    console.log("📌 LP info state after swap:");
    await fetchLpInfoState();
  });

  //
  // ------------------------------------------------
  // TEST #6: Swap some token for SOL
  // ------------------------------------------------
  it("Swaps some tokens for SOL", async () => {
    // Trader (lp) muốn gửi 50 tokens đến pool -> nhận lại SOL
    const tokenAmount = 50 * 10 ** 9;

    console.log("📌 Pool state before swap Token->SOL:");
    await fetchPoolState();

    await program.methods
      .swaptoken(new anchor.BN(tokenAmount))  // Tên hàm: swapToken
      .accountsPartial({
        pool: poolPda,
        trader: lp.publicKey,
        tokenMint: tokenMint,
        traderTokenAccount: userTokenAccount,
        tokenVault: tokenVault,
        solVault: solVaultPda,
      })
      .signers([lp])
      .rpc();

    console.log(`✅ Swapped ${tokenAmount} tokens for some SOL!`);
    console.log("📌 Pool state after swap Token->SOL:");
    await fetchPoolState();
    console.log("📌 LP info state after swap:");
    await fetchLpInfoState();
  });
});
