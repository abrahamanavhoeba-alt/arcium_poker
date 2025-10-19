import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArciumPoker } from "../target/types/arcium_poker";
import { expect } from "chai";

describe("Game Initialization Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ArciumPoker as Program<ArciumPoker>;
  const authority = provider.wallet;

  describe("Valid Game Creation", () => {
    it("Creates game with default parameters", async () => {
      const gameId = Date.now();
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("game"),
          authority.publicKey.toBuffer(),
          new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)
        ],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          null, // small_blind
          null, // big_blind
          null, // min_buy_in
          null, // max_buy_in
          null  // max_players
        )
        .accounts({
          authority: authority.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      expect(game.gameId.toNumber()).to.equal(gameId);
      expect(game.authority.toString()).to.equal(authority.publicKey.toString());
      expect(game.playerCount).to.equal(0);
    });

    it("Creates game with custom parameters", async () => {
      const gameId = Date.now() + 1;
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("game"),
          authority.publicKey.toBuffer(),
          new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)
        ],
        program.programId
      );

      const smallBlind = new anchor.BN(50);
      const bigBlind = new anchor.BN(100);
      const minBuyIn = new anchor.BN(5000);
      const maxBuyIn = new anchor.BN(10000);
      const maxPlayers = 6;

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          smallBlind,
          bigBlind,
          minBuyIn,
          maxBuyIn,
          maxPlayers
        )
        .accounts({
          authority: authority.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      expect(game.smallBlind.toNumber()).to.equal(50);
      expect(game.bigBlind.toNumber()).to.equal(100);
      expect(game.minBuyIn.toNumber()).to.equal(5000);
      expect(game.maxBuyIn.toNumber()).to.equal(10000);
      expect(game.maxPlayers).to.equal(6);
    });
  });

  describe("Invalid Game Creation", () => {
    it("Fails when big blind <= small blind", async () => {
      const gameId = Date.now() + 2;
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("game"),
          authority.publicKey.toBuffer(),
          new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)
        ],
        program.programId
      );

      try {
        await program.methods
          .initializeGame(
            new anchor.BN(gameId),
            new anchor.BN(100), // small blind
            new anchor.BN(100), // big blind same as small
            null,
            null,
            null
          )
          .accounts({
            authority: authority.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidGameConfig");
      }
    });

    it("Fails when max_buy_in < min_buy_in", async () => {
      const gameId = Date.now() + 3;
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("game"),
          authority.publicKey.toBuffer(),
          new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)
        ],
        program.programId
      );

      try {
        await program.methods
          .initializeGame(
            new anchor.BN(gameId),
            new anchor.BN(50),
            new anchor.BN(100),
            new anchor.BN(10000), // min
            new anchor.BN(5000),  // max < min
            null
          )
          .accounts({
            authority: authority.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidGameConfig");
      }
    });

    it("Fails when max_players > MAX_PLAYERS", async () => {
      const gameId = Date.now() + 4;
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("game"),
          authority.publicKey.toBuffer(),
          new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)
        ],
        program.programId
      );

      try {
        await program.methods
          .initializeGame(
            new anchor.BN(gameId),
            null,
            null,
            null,
            null,
            10 // MAX_PLAYERS is 6
          )
          .accounts({
            authority: authority.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidGameConfig");
      }
    });

    it("Fails when min_buy_in < 50 big blinds", async () => {
      const gameId = Date.now() + 5;
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("game"),
          authority.publicKey.toBuffer(),
          new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)
        ],
        program.programId
      );

      try {
        await program.methods
          .initializeGame(
            new anchor.BN(gameId),
            new anchor.BN(50),
            new anchor.BN(100),
            new anchor.BN(1000), // Only 10 BBs
            null,
            null
          )
          .accounts({
            authority: authority.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidGameConfig");
      }
    });
  });

  describe("Duplicate Game Prevention", () => {
    it("Fails when creating game with same ID", async () => {
      const gameId = Date.now() + 6;
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("game"),
          authority.publicKey.toBuffer(),
          new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)
        ],
        program.programId
      );

      // Create first game
      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          null, null, null, null, null
        )
        .accounts({
          authority: authority.publicKey,
        })
        .rpc();

      // Try to create duplicate
      try {
        await program.methods
          .initializeGame(
            new anchor.BN(gameId),
            null, null, null, null, null
          )
          .accounts({
            authority: authority.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        // Account already exists error
        expect(error).to.exist;
      }
    });
  });
});
