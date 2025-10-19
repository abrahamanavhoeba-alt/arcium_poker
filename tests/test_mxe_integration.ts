import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArciumPoker } from "../target/types/arcium_poker";
import { expect } from "chai";

/**
 * MXE Integration Tests
 * 
 * These tests demonstrate how to integrate with Arcium's MXE program
 * for real Multi-Party Computation (MPC).
 * 
 * NOTE: These tests require:
 * 1. Deployed MXE program (encrypted-ixs)
 * 2. MXE program ID
 * 3. Computation definition PDAs
 * 4. Mempool and cluster accounts
 * 
 * For development/testing without MXE deployment, the program
 * automatically falls back to mock mode.
 */

describe("MXE Integration Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ArciumPoker as Program<ArciumPoker>;

  // MXE Program Configuration
  // Replace these with actual values after deploying encrypted-ixs
  const MXE_PROGRAM_ID = new anchor.web3.PublicKey(
    "11111111111111111111111111111111" // Placeholder
  );

  let gamePda: anchor.web3.PublicKey;
  let gameId: number;

  beforeEach(async () => {
    gameId = Date.now();
    [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("game"),
        provider.wallet.publicKey.toBuffer(),
        new anchor.BN(gameId).toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    // Initialize game
    await program.methods
      .initializeGame(
        new anchor.BN(gameId),
        new anchor.BN(10), // Small blind
        new anchor.BN(20), // Big blind
        new anchor.BN(1000), // Min buy-in
        new anchor.BN(50000), // Max buy-in
        6 // Max players
      )
      .accounts({
        authority: provider.wallet.publicKey,
      })
      .rpc();
  });

  describe("Mock Mode (No MXE)", () => {
    it("Shuffles deck without MXE accounts", async () => {
      // Create and join players
      const players: anchor.web3.Keypair[] = [];
      const playerStates: anchor.web3.PublicKey[] = [];

      for (let i = 0; i < 3; i++) {
        const player = anchor.web3.Keypair.generate();
        players.push(player);

        // Airdrop SOL
        await provider.connection.confirmTransaction(
          await provider.connection.requestAirdrop(
            player.publicKey,
            2 * anchor.web3.LAMPORTS_PER_SOL
          )
        );

        // Join game
        await program.methods
          .joinGame(new anchor.BN(10000))
          .accounts({
            game: gamePda,
            player: player.publicKey,
          })
          .signers([player])
          .rpc();

        // Get player state PDA
        const [playerStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("player"),
            gamePda.toBuffer(),
            player.publicKey.toBuffer(),
          ],
          program.programId
        );
        playerStates.push(playerStatePda);
      }

      // Start game (without MXE accounts = mock mode)
      // Note: We don't pass remaining accounts - the Rust code handles this
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Verify game started
      const gameAccount = await program.account.game.fetch(gamePda);
      expect(gameAccount.stage).to.deep.equal({ preFlop: {} });
      expect(gameAccount.deckInitialized).to.be.true;

      console.log("✅ Mock shuffle completed successfully");
      console.log("   Game stage:", gameAccount.stage);
      console.log("   Deck initialized:", gameAccount.deckInitialized);
    });
  });

  describe("MXE Mode (With MXE Accounts)", () => {
    it.skip("Shuffles deck with MXE program", async () => {
      // This test is skipped until MXE program is deployed
      // To enable:
      // 1. Deploy encrypted-ixs as MXE program
      // 2. Update MXE_PROGRAM_ID above
      // 3. Create computation definition PDA
      // 4. Create mempool and cluster accounts
      // 5. Remove .skip from this test

      const player1 = anchor.web3.Keypair.generate();
      const player2 = anchor.web3.Keypair.generate();

      // ... (join players - same as above)

      // Derive MXE accounts
      const [compDef] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("comp_def"), Buffer.from("shuffle_deck")],
        MXE_PROGRAM_ID
      );

      const [mempool] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("mempool")],
        MXE_PROGRAM_ID
      );

      const [cluster] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("cluster")],
        MXE_PROGRAM_ID
      );

      // Start game WITH MXE accounts
      const playerEntropy = Array(2).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .remainingAccounts([
          // player states...
          // NOTE: MXE accounts would be added here when instruction is updated
        ])
        .rpc();

      console.log("✅ Real MPC shuffle queued");
      console.log("   Waiting for Arcium network callback...");
    });

    it.skip("Deals cards with MXE encryption", async () => {
      // Similar to above - requires MXE deployment
      console.log("TODO: Implement MXE card dealing test");
    });

    it.skip("Reveals cards at showdown with MXE", async () => {
      // Similar to above - requires MXE deployment
      console.log("TODO: Implement MXE card reveal test");
    });
  });

  describe("MXE Integration Documentation", () => {
    it("Shows how to use MXE in production", () => {
      console.log("\n📚 MXE Integration Guide:");
      console.log("================================\n");

      console.log("1. Deploy MXE Program:");
      console.log("   cd encrypted-ixs");
      console.log("   cargo build-sbf");
      console.log("   solana program deploy target/deploy/encrypted_ixs.so\n");

      console.log("2. Initialize MXE:");
      console.log("   arcium init-mxe --program-id <YOUR_MXE_ID>");
      console.log("   arcium init-cluster --name poker-cluster\n");

      console.log("3. Update Client Code:");
      console.log("   const mxeProgram = new PublicKey('YOUR_MXE_ID');");
      console.log("   // Pass mxeProgram to startGame() accounts\n");

      console.log("4. Generate Player Entropy:");
      console.log("   import { RescueCipher } from '@arcium-hq/client';");
      console.log("   const entropy = crypto.getRandomValues(new Uint8Array(32));\n");

      console.log("5. Handle Callbacks:");
      console.log("   // Set up webhook to receive MPC results");
      console.log("   // Update game state when shuffle/deal/reveal completes\n");

      console.log("✅ See PHASE_3_INTEGRATION_COMPLETE.md for details");
    });
  });
});
