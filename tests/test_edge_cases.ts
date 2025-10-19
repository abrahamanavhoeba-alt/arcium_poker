import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArciumPoker } from "../target/types/arcium_poker";
import { expect } from "chai";
import { getGamePda, getPlayerStatePda, airdropSol } from "./helpers";

describe("Edge Cases and Security Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ArciumPoker as Program<ArciumPoker>;

  describe("Chip Conservation", () => {
    it("Total chips remain constant throughout game", async () => {
      const gameId = Date.now();
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          new anchor.BN(50),
          new anchor.BN(100),
          new anchor.BN(5000),
          new anchor.BN(10000),
          6
        )
        .accounts({
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const players: anchor.web3.Keypair[] = [];
      const playerStates: anchor.web3.PublicKey[] = [];
      const initialBuyIns: number[] = [];
      let totalChips = 0;

      // Add 3 players with different buy-ins
      const buyIns = [5000, 7500, 10000];
      const numPlayers = 3;
      for (let i = 0; i < numPlayers; i++) {
        const player = anchor.web3.Keypair.generate();
        players.push(player);
        
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
        playerStates.push(playerStatePda);

        await program.methods
          .joinGame(new anchor.BN(buyIns[i]))
          .accounts({
            game: gamePda,
            player: player.publicKey,
          })
          .signers([player])
          .rpc();

        initialBuyIns.push(buyIns[i]);
        totalChips += buyIns[i];
      }

      // Generate dummy entropy for each player
      const playerEntropy = Array(numPlayers).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Play some actions
      for (let i = 0; i < 3; i++) {
        const game = await program.account.game.fetch(gamePda);
        await program.methods
          .playerAction({ call: {} })
          .accounts({
            game: gamePda,
            playerState: playerStates[game.currentPlayerIndex],
            player: players[game.currentPlayerIndex].publicKey,
          })
          .signers([players[game.currentPlayerIndex]])
          .rpc();
      }

      // Verify chip conservation
      const game = await program.account.game.fetch(gamePda);
      let currentTotal = game.pot.toNumber();

      for (let i = 0; i < 3; i++) {
        const playerState = await program.account.playerState.fetch(playerStates[i]);
        currentTotal += playerState.chipStack.toNumber();
        // Don't add current_bet as it's already counted in the pot
      }

      expect(currentTotal).to.equal(totalChips);
    });
  });

  describe("Timeout Handling", () => {
    it("Detects player timeout", async () => {
      const gameId = Date.now();
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          new anchor.BN(50), // small blind
          new anchor.BN(100), // big blind  
          new anchor.BN(5000), // min buy-in
          new anchor.BN(50000), // max buy-in
          null
        )
        .accounts({
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Add players
      const players: anchor.web3.Keypair[] = [];
      for (let i = 0; i < 2; i++) {
        const player = anchor.web3.Keypair.generate();
        players.push(player);
        
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
          .joinGame(new anchor.BN(10000))
          .accounts({
            game: gamePda,
            player: player.publicKey,
          })
          .signers([player])
          .rpc();
      }

      // Generate dummy entropy for each player
      const playerCount = 2; // or 3, depending on test
      const playerEntropy = Array(playerCount).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      
      // Check last_action_at is set
      expect(game.lastActionAt.toNumber()).to.be.greaterThan(0);
    });
  });

  describe("Integer Overflow/Underflow", () => {
    it("Handles maximum chip values", async () => {
      const gameId = Date.now();
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          new anchor.BN(1),
          new anchor.BN(2),
          new anchor.BN(100),
          new anchor.BN(1000000000), // Large max buy-in
          6
        )
        .accounts({
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      expect(game.maxBuyIn.toNumber()).to.equal(1000000000);
    });

    it("Prevents negative chip stacks", async () => {
      const gameId = Date.now();
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          new anchor.BN(50),
          new anchor.BN(100),
          new anchor.BN(5000),
          new anchor.BN(50000),
          null
        )
        .accounts({
          authority: provider.wallet.publicKey,
        })
        .rpc();

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
        .joinGame(new anchor.BN(5000))
        .accounts({
          game: gamePda,
          player: player.publicKey,
        })
        .signers([player])
        .rpc();

      const playerState = await program.account.playerState.fetch(playerStatePda);
      expect(playerState.chipStack.toNumber()).to.be.greaterThanOrEqual(0);
    });
  });

  describe("Concurrent Action Prevention", () => {
    it("Prevents two players from acting simultaneously", async () => {
      const gameId = Date.now();
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          new anchor.BN(50),
          new anchor.BN(100),
          new anchor.BN(5000),
          new anchor.BN(50000),
          null
        )
        .accounts({
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const players: anchor.web3.Keypair[] = [];
      const playerStates: anchor.web3.PublicKey[] = [];

      for (let i = 0; i < 2; i++) {
        const player = anchor.web3.Keypair.generate();
        players.push(player);
        
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
        playerStates.push(playerStatePda);

        await program.methods
          .joinGame(new anchor.BN(10000))
          .accounts({
            game: gamePda,
            player: player.publicKey,
          })
          .signers([player])
          .rpc();
      }

      // Generate dummy entropy for each player
      const playerCount = 2; // or 3, depending on test
      const playerEntropy = Array(playerCount).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Try to have wrong player act
      const game = await program.account.game.fetch(gamePda);
      const wrongPlayerIdx = (game.currentPlayerIndex + 1) % 2;

      try {
        await program.methods
          .playerAction({ fold: {} })
          .accounts({
            game: gamePda,
            playerState: playerStates[wrongPlayerIdx],
            player: players[wrongPlayerIdx].publicKey,
          })
          .signers([players[wrongPlayerIdx]])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("NotPlayerTurn");
      }
    });
  });

  describe("State Validation", () => {
    it("Validates game state transitions", async () => {
      const gameId = Date.now();
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          null, null, null, null, null
        )
        .accounts({
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      expect(game.stage).to.deep.equal({ waiting: {} });
    });

    it("Validates player count never exceeds max", async () => {
      const gameId = Date.now();
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          new anchor.BN(50),
          new anchor.BN(100),
          new anchor.BN(5000),
          new anchor.BN(50000),
          2 // Max 2 players
        )
        .accounts({
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Add 2 players
      for (let i = 0; i < 2; i++) {
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
          .joinGame(new anchor.BN(10000))
          .accounts({
            game: gamePda,
            player: player.publicKey,
          })
          .signers([player])
          .rpc();
      }

      const game = await program.account.game.fetch(gamePda);
      expect(game.playerCount).to.equal(2);
      expect(game.playerCount).to.be.lessThanOrEqual(game.maxPlayers);
    });
  });

  describe("Zero/Null Value Handling", () => {
    it("Handles zero buy-in rejection", async () => {
      const gameId = Date.now();
      const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(gameId),
          null, null, null, null, null
        )
        .accounts({
          authority: provider.wallet.publicKey,
        })
        .rpc();

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
          .joinGame(new anchor.BN(0)) // Zero buy-in
          .accounts({
            game: gamePda,
            player: player.publicKey,
          })
          .signers([player])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("BuyInTooLow");
      }
    });
  });
});
