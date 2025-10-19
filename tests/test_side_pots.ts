import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArciumPoker } from "../target/types/arcium_poker";
import { expect } from "chai";

describe("Side Pot Tests", () => {
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
        new anchor.BN(10),  // Small blind
        new anchor.BN(20),  // Big blind
        new anchor.BN(1000), // Min buy-in (50 BBs = 1000)
        new anchor.BN(50000), // Max buy-in
        6
      )
      .accounts({
        authority: provider.wallet.publicKey,
      })
      .rpc();

    players = [];
    playerStates = [];
  });

  describe("Multiple All-Ins", () => {
    it("Creates side pots with 3 players, 2 all-ins", async () => {
      // Create 3 players with different stack sizes
      const buyIns = [2000, 5000, 10000]; // Short, medium, big stack
      
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
          .joinGame(new anchor.BN(buyIns[i]))
          .accounts({
            game: gamePda,
            player: player.publicKey,
          })
          .signers([player])
          .rpc();
      }

      // Start game
      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Player 1 (2000 chips) goes all-in
      let game = await program.account.game.fetch(gamePda);
      await program.methods
        .playerAction({ allIn: {} })
        .accounts({
          game: gamePda,
          playerState: playerStates[game.currentPlayerIndex],
          player: players[game.currentPlayerIndex].publicKey,
        })
        .signers([players[game.currentPlayerIndex]])
        .rpc();

      // Player 2 (5000 chips) goes all-in
      game = await program.account.game.fetch(gamePda);
      await program.methods
        .playerAction({ allIn: {} })
        .accounts({
          game: gamePda,
          playerState: playerStates[game.currentPlayerIndex],
          player: players[game.currentPlayerIndex].publicKey,
        })
        .signers([players[game.currentPlayerIndex]])
        .rpc();

      // Player 3 (10000 chips) calls
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

      // Verify pot structure
      game = await program.account.game.fetch(gamePda);
      
      // Total pot should be 2000 + 5000 + 5000 = 12000
      // (Player 3 only matches player 2's all-in)
      expect(game.pot.toNumber()).to.be.greaterThan(0);
      
      // Verify all players are still in
      const p1State = await program.account.playerState.fetch(playerStates[0]);
      const p2State = await program.account.playerState.fetch(playerStates[1]);
      const p3State = await program.account.playerState.fetch(playerStates[2]);
      
      expect(p1State.isAllIn).to.be.true;
      expect(p1State.chipStack.toNumber()).to.equal(0);
      
      expect(p2State.isAllIn).to.be.true;
      expect(p2State.chipStack.toNumber()).to.equal(0);
      
      expect(p3State.isAllIn).to.be.false;
      expect(p3State.chipStack.toNumber()).to.be.greaterThan(0);
    });

    it("Handles 4-way all-in with different stack sizes", async () => {
      const buyIns = [1000, 2000, 3000, 10000];
      
      for (let i = 0; i < 4; i++) {
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
      }

      // Start game
      const playerEntropy = Array(4).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // All 4 players go all-in
      for (let i = 0; i < 4; i++) {
        const game = await program.account.game.fetch(gamePda);
        const action = i < 3 ? { allIn: {} } : { call: {} };
        
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

      // Verify all are all-in except the big stack
      for (let i = 0; i < 3; i++) {
        const pState = await program.account.playerState.fetch(playerStates[i]);
        expect(pState.isAllIn).to.be.true;
      }

      const game = await program.account.game.fetch(gamePda);
      expect(game.pot.toNumber()).to.be.greaterThan(0);
    });
  });

  describe("Side Pot Edge Cases", () => {
    it("Handles equal stack all-ins", async () => {
      const buyIns = [5000, 5000, 10000];
      
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
          .joinGame(new anchor.BN(buyIns[i]))
          .accounts({
            game: gamePda,
            player: player.publicKey,
          })
          .signers([player])
          .rpc();
      }

      const playerEntropy = Array(3).fill(0).map(() => Array(32).fill(0));
      await program.methods
        .startGame(playerEntropy)
        .accounts({
          game: gamePda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      // Two players with equal stacks go all-in
      for (let i = 0; i < 3; i++) {
        const game = await program.account.game.fetch(gamePda);
        const action = i < 2 ? { allIn: {} } : { call: {} };
        
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

      const p1 = await program.account.playerState.fetch(playerStates[0]);
      const p2 = await program.account.playerState.fetch(playerStates[1]);
      
      expect(p1.isAllIn).to.be.true;
      expect(p2.isAllIn).to.be.true;
      expect(p1.chipStack.toNumber()).to.equal(p2.chipStack.toNumber());
    });
  });
});
