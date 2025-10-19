import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArciumPoker } from "../target/types/arcium_poker";
import { expect } from "chai";

describe("Game Flow Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ArciumPoker as Program<ArciumPoker>;
  
  let gamePda: anchor.web3.PublicKey;
  let gameId: number;
  let players: anchor.web3.Keypair[];
  let playerStates: anchor.web3.PublicKey[];

  beforeEach(async () => {
    gameId = Date.now();
    [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
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
        game: gamePda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    players = [];
    playerStates = [];
    
    for (let i = 0; i < 3; i++) {
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
          playerState: playerStatePda,
          player: player.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player])
        .rpc();
    }
  });

  describe("Game Start", () => {
    it("Starts game with minimum players", async () => {
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      expect(game.stage).to.deep.equal({ preFlop: {} });
      expect(game.deckInitialized).to.be.true;
    });

    it("Fails when not enough players", async () => {
      // Create new game with only 1 player
      const newGameId = Date.now() + 1;
      const [newGamePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(newGameId).toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      await program.methods
        .initializeGame(
          new anchor.BN(newGameId),
          new anchor.BN(50),
          new anchor.BN(100),
          new anchor.BN(5000),
          new anchor.BN(50000),
          null
        )
        .accounts({
          game: newGamePda,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
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
          newGamePda.toBuffer(),
          player.publicKey.toBuffer(),
        ],
        program.programId
      );

      await program.methods
        .joinGame(new anchor.BN(10000))
        .accounts({
          game: newGamePda,
          playerState: playerStatePda,
          player: player.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player])
        .rpc();

      try {
        const playerEntropy2 = Array(1).fill(0).map(() => Array(32).fill(0));
        await program.methods
          .startGame(playerEntropy2)
          .accounts({
            game: newGamePda,
            authority: provider.wallet.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("NotEnoughPlayers");
      }
    });

    it("Fails when game already started", async () => {
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      try {
        const playerEntropy2 = Array(3).fill(0).map(() => Array(32).fill(0));
        await program.methods
          .startGame(playerEntropy2)
          .accounts({
            game: gamePda,
            authority: provider.wallet.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("GameAlreadyStarted");
      }
    });

    it("Fails when non-authority tries to start", async () => {
      const nonAuthority = anchor.web3.Keypair.generate();
      const signature = await provider.connection.requestAirdrop(
        nonAuthority.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);

      try {
        const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
        await program.methods
          .startGame(playerEntropy)
          .accounts({
            game: gamePda,
            authority: nonAuthority.publicKey,
          })
          .signers([nonAuthority])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Stage Transitions", () => {
    beforeEach(async () => {
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();
    });

    it("Transitions from PreFlop to Flop", async () => {
      // Complete pre-flop betting
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

      const game = await program.account.game.fetch(gamePda);
      expect(game.stage).to.deep.equal({ flop: {} });
      expect(game.communityCardsRevealed).to.equal(3);
    });

    it("Transitions from Flop to Turn", async () => {
      // Complete pre-flop
      for (let i = 0; i < 3; i++) {
        let game = await program.account.game.fetch(gamePda);
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

      // Complete flop betting
      for (let i = 0; i < 3; i++) {
        let game = await program.account.game.fetch(gamePda);
        await program.methods
          .playerAction({ check: {} })
          .accounts({
            game: gamePda,
            playerState: playerStates[game.currentPlayerIndex],
            player: players[game.currentPlayerIndex].publicKey,
          })
          .signers([players[game.currentPlayerIndex]])
          .rpc();
      }

      const game = await program.account.game.fetch(gamePda);
      expect(game.stage).to.deep.equal({ turn: {} });
      expect(game.communityCardsRevealed).to.equal(4);
    });

    it("Transitions from Turn to River", async () => {
      // Complete pre-flop, flop, and turn
      for (let round = 0; round < 3; round++) {
        for (let i = 0; i < 3; i++) {
          let game = await program.account.game.fetch(gamePda);
          const action = round === 0 ? { call: {} } : { check: {} };
          await program.methods
            .playerAction(action)
            .accounts({
              game: gamePda,
              playerState: playerStates[game.currentPlayerIndex],
              player: players[game.currentPlayerIndex].publicKey,
            })
            .signers([players[game.currentPlayerIndex]])
            .rpc();
        }
      }

      const game = await program.account.game.fetch(gamePda);
      expect(game.stage).to.deep.equal({ river: {} });
      expect(game.communityCardsRevealed).to.equal(5);
    });

    it("Transitions to Showdown after River", async () => {
      // Complete all betting rounds
      for (let round = 0; round < 4; round++) {
        for (let i = 0; i < 3; i++) {
          let game = await program.account.game.fetch(gamePda);
          const action = round === 0 ? { call: {} } : { check: {} };
          await program.methods
            .playerAction(action)
            .accounts({
              game: gamePda,
              playerState: playerStates[game.currentPlayerIndex],
              player: players[game.currentPlayerIndex].publicKey,
            })
            .signers([players[game.currentPlayerIndex]])
            .rpc();
        }
      }

      const game = await program.account.game.fetch(gamePda);
      expect(game.stage).to.deep.equal({ showdown: {} });
    });
  });

  describe("Early Game End", () => {
    beforeEach(async () => {
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();
    });

    it("Ends game when all but one player folds", async () => {
      // First two players fold
      for (let i = 0; i < 2; i++) {
        const game = await program.account.game.fetch(gamePda);
        await program.methods
          .playerAction({ fold: {} })
          .accounts({
            game: gamePda,
            playerState: playerStates[game.currentPlayerIndex],
            player: players[game.currentPlayerIndex].publicKey,
          })
          .signers([players[game.currentPlayerIndex]])
          .rpc();
      }

      const game = await program.account.game.fetch(gamePda);
      expect(game.stage).to.deep.equal({ finished: {} });
    });
  });

  describe("New Hand", () => {
    it("Starts new hand after previous completes", async () => {
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Complete a hand (all fold except one)
      for (let i = 0; i < 2; i++) {
        const game = await program.account.game.fetch(gamePda);
        await program.methods
          .playerAction({ fold: {} })
          .accounts({
            game: gamePda,
            playerState: playerStates[game.currentPlayerIndex],
            player: players[game.currentPlayerIndex].publicKey,
          })
          .signers([players[game.currentPlayerIndex]])
          .rpc();
      }

      // Start new hand
      await program.methods
        .newHand()
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      expect(game.stage).to.deep.equal({ preFlop: {} });
      expect(game.pot.toNumber()).to.equal(0);
    });
  });

  describe("End Game", () => {
    it("Authority ends game", async () => {
      await program.methods
        .endGame()
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      expect(game.stage).to.deep.equal({ finished: {} });
    });

    it("Fails when non-authority tries to end", async () => {
      try {
        await program.methods
          .endGame()
          .accounts({
            game: gamePda,
            authority: players[0].publicKey,
          })
          .signers([players[0]])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });
});
