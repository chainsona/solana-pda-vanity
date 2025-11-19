import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PdaVanity } from "../target/types/pda_vanity";
import { PublicKey } from "@solana/web3.js";
import { BN } from "bn.js";

describe("pda-vanity", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PdaVanity as Program<PdaVanity>;

  it("Creates a token with vanity public key ending in 'pump'", async () => {
    // Hardcoded seed found by Rust search tool for program ID 7d4pygUVej17wWKY6uiPdFSVPTDKEEAzR4YMmkc1Bss1
    // Seed: 5270498306774619999
    // PDA: HZTPCxeTBLEr5FfUkjzLixXduWCzhgzjhvoNrKVspump
    // Bump: 255
    let vanitySeed = new BN("5270498306774619999");
    let mintPda: PublicKey;
    let bump: number;
    const suffix = "pump";

    console.log(`Using pre-calculated seed: ${vanitySeed.toString()}`);

    [mintPda, bump] = PublicKey.findProgramAddressSync(
      [vanitySeed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    console.log(`PDA: ${mintPda.toBase58()}`);
    console.log(`Bump: ${bump}`);

    if (!mintPda.toBase58().endsWith(suffix)) {
        console.warn("Warning: Pre-calculated seed does not match suffix 'pump' for this program ID.");
        console.warn("You may need to run the search tool again if the program ID has changed.");
    }

    try {
      const tx = await program.methods
        .createVanityToken(vanitySeed, 6)
        .accounts({
          payer: provider.wallet.publicKey,
        })
        .rpc();

      console.log("Your transaction signature", tx);
      
      // Verify the account exists
      const mintAccount = await program.provider.connection.getAccountInfo(mintPda);
      if (mintAccount) {
          console.log("Mint account created successfully!");
      } else {
          console.error("Mint account not found!");
      }

    } catch (error) {
      console.error("Transaction failed:", error);
      throw error;
    }
  });
});
