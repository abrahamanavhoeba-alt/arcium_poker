import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArciumPoker } from "../target/types/arcium_poker";

describe("arcium_poker", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ArciumPoker as Program<ArciumPoker>;

  it("Is initialized!", async () => {
    const gameId = Date.now();
    const [gamePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("game"), provider.wallet.publicKey.toBuffer(), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const tx = await program.methods
      .initializeGame(
        new anchor.BN(gameId),
        null, null, null, null, null
      )
      .accounts({
        authority: provider.wallet.publicKey,
      })
      .rpc();
    
    console.log("Your transaction signature", tx);
    
    const game = await program.account.game.fetch(gamePda);
    console.log("Game initialized with ID:", game.gameId.toString());
  });
});
