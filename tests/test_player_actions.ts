import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArciumPoker } from "../target/types/arcium_poker";
import { expect } from "chai";
import { getGamePda, getPlayerStatePda, airdropSol } from "./helpers";

describe("Player Actions Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ArciumPoker as Program<ArciumPoker>;
  
  let gamePda: anchor.web3.PublicKey;
  let gameId: number;

  beforeEach(async () => {
    // Create a new game for each test
    gameId = Date.now();
    [gamePda] = getGamePda(program.programId, provider.wallet.publicKey, gameId);

    await program.methods
      .initializeGame(
        new anchor.BN(gameId),
        new anchor.BN(50),   // small blind
        new anchor.BN(100),  // big blind
        new anchor.BN(5000), // min buy-in
        new anchor.BN(10000), // max buy-in
        6 // max players
      )
      .accounts({
        game: gamePda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  describe("Player Join", () => {
    it("Player joins with valid buy-in", async () => {
      const player = anchor.web3.Keypair.generate();
      
      // Airdrop SOL to player
      await airdropSol(provider.connection, player.publicKey);

      const [playerStatePda] = getPlayerStatePda(
        program.programId,
        gamePda,
        player.publicKey
      );

      const buyIn = new anchor.BN(7500);

      await program.methods
        .joinGame(buyIn)
        .accounts({
          game: gamePda,
          player: player.publicKey,
        })
        .signers([player])
        .rpc();

      const playerState = await program.account.playerState.fetch(playerStatePda);
      expect(playerState.chipStack.toNumber()).to.equal(7500);
      expect(playerState.player.toString()).to.equal(player.publicKey.toString());

      const game = await program.account.game.fetch(gamePda);
      expect(game.playerCount).to.equal(1);
    });

    it("Fails when buy-in < min_buy_in", async () => {
      const player = anchor.web3.Keypair.generate();
      
      const signature = await provider.connection.requestAirdrop(
        player.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);

      const [playerStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("player"),
          gamePda.toBuffer(),
          player.publicKey.toBuffer(),
        ],
        program.programId
      );

      try {
        await program.methods
          .joinGame(new anchor.BN(1000)) // Less than min_buy_in (5000)
          .accounts({
            game: gamePda,
            playerState: playerStatePda,
            player: player.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("BuyInTooLow");
      }
    });

    it("Fails when buy-in > max_buy_in", async () => {
      const player = anchor.web3.Keypair.generate();
      
      const signature = await provider.connection.requestAirdrop(
        player.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);

      const [playerStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("player"),
          gamePda.toBuffer(),
          player.publicKey.toBuffer(),
        ],
        program.programId
      );

      try {
        await program.methods
          .joinGame(new anchor.BN(15000)) // More than max_buy_in (10000)
          .accounts({
            game: gamePda,
            playerState: playerStatePda,
            player: player.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("BuyInTooHigh");
      }
    });

    it("Fails when game is full", async () => {
      // Fill the game with max players (6)
      for (let i = 0; i < 6; i++) {
        const player = anchor.web3.Keypair.generate();
        
        const signature = await provider.connection.requestAirdrop(
          player.publicKey,
          2 * anchor.web3.LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(signature);

        const [playerStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("player"),
            gamePda.toBuffer(),
            player.publicKey.toBuffer(),
          ],
          program.programId
        );

        await program.methods
          .joinGame(new anchor.BN(7500))
          .accounts({
            game: gamePda,
            playerState: playerStatePda,
            player: player.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([player])
          .rpc();
      }

      // Try to add 7th player
      const extraPlayer = anchor.web3.Keypair.generate();
      const signature = await provider.connection.requestAirdrop(
        extraPlayer.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);

      const [playerStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("player"),
          gamePda.toBuffer(),
          extraPlayer.publicKey.toBuffer(),
        ],
        program.programId
      );

      try {
        await program.methods
          .joinGame(new anchor.BN(7500))
          .accounts({
            game: gamePda,
            playerState: playerStatePda,
            player: extraPlayer.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([extraPlayer])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("GameFull");
      }
    });

    it("Fails when player tries to join twice", async () => {
      const player = anchor.web3.Keypair.generate();
      
      const signature = await provider.connection.requestAirdrop(
        player.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);

      const [playerStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("player"),
          gamePda.toBuffer(),
          player.publicKey.toBuffer(),
        ],
        program.programId
      );

      // First join
      await program.methods
        .joinGame(new anchor.BN(7500))
        .accounts({
          game: gamePda,
          playerState: playerStatePda,
          player: player.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player])
        .rpc();

      // Try to join again
      try {
        await program.methods
          .joinGame(new anchor.BN(7500))
          .accounts({
            game: gamePda,
            playerState: playerStatePda,
            player: player.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        // Account already exists
        expect(error).to.exist;
      }
    });
  });

  describe("Player Leave", () => {
    let player: anchor.web3.Keypair;
    let playerStatePda: anchor.web3.PublicKey;

    beforeEach(async () => {
      player = anchor.web3.Keypair.generate();
      
      const signature = await provider.connection.requestAirdrop(
        player.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);

      [playerStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("player"),
          gamePda.toBuffer(),
          player.publicKey.toBuffer(),
        ],
        program.programId
      );

      await program.methods
        .joinGame(new anchor.BN(7500))
        .accounts({
          game: gamePda,
          playerState: playerStatePda,
          player: player.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player])
        .rpc();
    });

    it("Player leaves successfully", async () => {
      await program.methods
        .leaveGame()
        .accounts({
          game: gamePda,
          player: player.publicKey,
        })
        .signers([player])
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      expect(game.playerCount).to.equal(0);
    });

    it("Fails when non-player tries to leave", async () => {
      const nonPlayer = anchor.web3.Keypair.generate();
      
      const signature = await provider.connection.requestAirdrop(
        nonPlayer.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);

      try {
        await program.methods
          .leaveGame()
          .accounts({
            game: gamePda,
            playerState: playerStatePda, // Using wrong player state
            player: nonPlayer.publicKey,
          })
          .signers([nonPlayer])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });
});
