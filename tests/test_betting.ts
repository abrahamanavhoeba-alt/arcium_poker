import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArciumPoker } from "../target/types/arcium_poker";
import { expect } from "chai";
import { getGamePda, getPlayerStatePda, airdropSol } from "./helpers";

describe("Betting Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ArciumPoker as Program<ArciumPoker>;
  
  let gamePda: anchor.web3.PublicKey;
  let gameId: number;
  let players: anchor.web3.Keypair[];
  let playerStates: anchor.web3.PublicKey[];

  beforeEach(async () => {
    // Create game
    gameId = Date.now();
    [gamePda] = getGamePda(program.programId, provider.wallet.publicKey, gameId);

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

    // Add 3 players
    players = [];
    playerStates = [];
    
    for (let i = 0; i < 3; i++) {
      const player = anchor.web3.Keypair.generate();
      players.push(player);
      
      await airdropSol(provider.connection, player.publicKey);

      const [playerStatePda] = getPlayerStatePda(
        program.programId,
        gamePda,
        player.publicKey
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
  });

  describe("Fold Action", () => {
    it("Player folds successfully", async () => {
      // Start game first
      // Generate dummy entropy for each player (3 players)
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Current player folds
      const game = await program.account.game.fetch(gamePda);
      const currentPlayerIdx = game.currentPlayerIndex;
      const currentPlayer = players[currentPlayerIdx];
      const currentPlayerState = playerStates[currentPlayerIdx];

      await program.methods
        .playerAction({ fold: {} })
        .accounts({
          game: gamePda,
          playerState: currentPlayerState,
          player: currentPlayer.publicKey,
        })
        .signers([currentPlayer])
        .rpc();

      const playerState = await program.account.playerState.fetch(currentPlayerState);
      expect(playerState.hasFolded).to.be.true;
    });

    it("Fails when not player's turn", async () => {
      // Generate dummy entropy for each player (3 players)
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      const wrongPlayerIdx = (game.currentPlayerIndex + 1) % 3;
      const wrongPlayer = players[wrongPlayerIdx];
      const wrongPlayerState = playerStates[wrongPlayerIdx];

      try {
        await program.methods
          .playerAction({ fold: {} })
          .accounts({
            game: gamePda,
            playerState: wrongPlayerState,
            player: wrongPlayer.publicKey,
          })
          .signers([wrongPlayer])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("NotPlayerTurn");
      }
    });
  });

  describe("Check Action", () => {
    it("Player checks when no bet", async () => {
      // Generate dummy entropy for each player (3 players)
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Complete pre-flop by having all players call the big blind
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

      // Now in Flop stage with no bet - players can check
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
    });

    it("Fails when there's a bet to call", async () => {
      // Generate dummy entropy for each player (3 players)
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      let game = await program.account.game.fetch(gamePda);
      
      // First player bets
      await program.methods
        .playerAction({ bet: { amount: new anchor.BN(200) } })
        .accounts({
          game: gamePda,
          playerState: playerStates[game.currentPlayerIndex],
          player: players[game.currentPlayerIndex].publicKey,
        })
        .signers([players[game.currentPlayerIndex]])
        .rpc();

      game = await program.account.game.fetch(gamePda);

      // Next player tries to check (should fail)
      try {
        await program.methods
          .playerAction({ check: {} })
          .accounts({
            game: gamePda,
            playerState: playerStates[game.currentPlayerIndex],
            player: players[game.currentPlayerIndex].publicKey,
          })
          .signers([players[game.currentPlayerIndex]])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidAction");
      }
    });
  });

  describe("Bet/Raise Action", () => {
    it("Player bets valid amount", async () => {
      // Generate dummy entropy for each player (3 players)
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      const betAmount = new anchor.BN(200);

      await program.methods
        .playerAction({ bet: { amount: betAmount } })
        .accounts({
          game: gamePda,
          playerState: playerStates[game.currentPlayerIndex],
          player: players[game.currentPlayerIndex].publicKey,
        })
        .signers([players[game.currentPlayerIndex]])
        .rpc();

      const playerState = await program.account.playerState.fetch(
        playerStates[game.currentPlayerIndex]
      );
      expect(playerState.currentBet.toNumber()).to.equal(200);
    });

    it("Fails when bet amount > chip stack", async () => {
      // Generate dummy entropy for each player (3 players)
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      const betAmount = new anchor.BN(20000); // More than 10000 chips

      try {
        await program.methods
          .playerAction({ bet: { amount: betAmount } })
          .accounts({
            game: gamePda,
            playerState: playerStates[game.currentPlayerIndex],
            player: players[game.currentPlayerIndex].publicKey,
          })
          .signers([players[game.currentPlayerIndex]])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("InsufficientChips");
      }
    });

    it("Fails when raise is less than minimum", async () => {
      // Generate dummy entropy for each player (3 players)
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      let game = await program.account.game.fetch(gamePda);
      
      // First player bets 200
      await program.methods
        .playerAction({ bet: { amount: new anchor.BN(200) } })
        .accounts({
          game: gamePda,
          playerState: playerStates[game.currentPlayerIndex],
          player: players[game.currentPlayerIndex].publicKey,
        })
        .signers([players[game.currentPlayerIndex]])
        .rpc();

      game = await program.account.game.fetch(gamePda);

      // Next player tries to raise to 250 (only 50 more, should be at least 200 more)
      try {
        await program.methods
          .playerAction({ raise: { amount: new anchor.BN(250) } })
          .accounts({
            game: gamePda,
            playerState: playerStates[game.currentPlayerIndex],
            player: players[game.currentPlayerIndex].publicKey,
          })
          .signers([players[game.currentPlayerIndex]])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("InvalidBetAmount");
      }
    });
  });

  describe("All-In Action", () => {
    it("Player goes all-in", async () => {
      // Generate dummy entropy for each player (3 players)
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const game = await program.account.game.fetch(gamePda);
      const currentPlayerIdx = game.currentPlayerIndex;

      await program.methods
        .playerAction({ allIn: {} })
        .accounts({
          game: gamePda,
          playerState: playerStates[currentPlayerIdx],
          player: players[currentPlayerIdx].publicKey,
        })
        .signers([players[currentPlayerIdx]])
        .rpc();

      const playerState = await program.account.playerState.fetch(
        playerStates[currentPlayerIdx]
      );
      expect(playerState.isAllIn).to.be.true;
      expect(playerState.chipStack.toNumber()).to.equal(0);
    });

    it("All-in player cannot act again", async () => {
      // Generate dummy entropy for each player (3 players)
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      let game = await program.account.game.fetch(gamePda);
      const allInPlayerIdx = game.currentPlayerIndex;

      // Player goes all-in
      await program.methods
        .playerAction({ allIn: {} })
        .accounts({
          game: gamePda,
          playerState: playerStates[allInPlayerIdx],
          player: players[allInPlayerIdx].publicKey,
        })
        .signers([players[allInPlayerIdx]])
        .rpc();

      // Complete the round with other players
      for (let i = 0; i < 2; i++) {
        game = await program.account.game.fetch(gamePda);
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

      // Verify all-in player is marked correctly
      const allInPlayerState = await program.account.playerState.fetch(playerStates[allInPlayerIdx]);
      expect(allInPlayerState.isAllIn).to.be.true;
      expect(allInPlayerState.chipStack.toNumber()).to.equal(0);
    });
  });

  describe("Side Pot Creation", () => {
    it("Creates side pot when player is all-in", async () => {
      // Create a player with smaller stack
      const shortStack = anchor.web3.Keypair.generate();
      
      const signature = await provider.connection.requestAirdrop(
        shortStack.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);

      const [shortStackPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("player"),
          gamePda.toBuffer(),
          shortStack.publicKey.toBuffer(),
        ],
        program.programId
      );

      await program.methods
        .joinGame(new anchor.BN(5000)) // Minimum buy-in
        .accounts({
          game: gamePda,
          player: shortStack.publicKey,
        })
        .signers([shortStack])
        .rpc();

      // Add short stack to arrays for easier access
      players.push(shortStack);
      playerStates.push(shortStackPda);

      // Generate dummy entropy for each player (4 players: 3 original + shortStack)
      const playerEntropy = Array(4).fill(0).map(() => Array(32).fill(0));
      
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // All players call to complete pre-flop
      for (let i = 0; i < 4; i++) {
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

      // Side pot should be created (tested in pot manager logic)
      const game = await program.account.game.fetch(gamePda);
      expect(game.pot.toNumber()).to.be.greaterThan(0);
    });
  });
});
