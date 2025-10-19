# Arcium MPC Integration Guide for Poker

## Overview
This guide explains how to integrate real Arcium MPC functionality into the poker smart contract.

## Architecture

### 1. Arcium Components

#### MXE (Multi-party Execution Environment)
- Deployed Solana program that handles confidential computations
- Coordinates with Arcium network nodes
- Manages encrypted state

#### Arcis Framework
- Rust DSL for writing confidential instructions
- Extends Anchor framework
- Mark functions as `#[confidential]` for MPC execution

#### Client SDK (`@arcium-hq/arcium-sdk`)
- TypeScript library for encrypting/decrypting data
- RescueCipher for encryption
- x25519 key exchange

### 2. Encryption Flow

```
Client Side:
1. Generate x25519 keypair
2. Encrypt data using RescueCipher
3. Send encrypted data to Solana program

Solana Program:
4. Pass encrypted data to MXE
5. MXE coordinates with Arcium nodes

Arcium Network:
6. Nodes perform MPC computation
7. Return encrypted result

Client Side:
8. Decrypt result using RescueCipher
```

### 3. Key Concepts

#### Enc<Owner, T>
- `Enc<Shared, T>`: Data shared among all MPC nodes
- `Enc<Mxe, T>`: Data owned by MXE
- `Enc<Owner, T>`: Data encrypted to specific owner

#### Encryption Types
- **Rescue Cipher**: Custom cipher over finite field F_p (p = 2^255 - 19)
- **CTR Mode**: Counter mode with nonce
- **x25519**: Key exchange protocol
- **HKDF**: Key derivation

## Implementation for Poker

### Step 1: Install Arcium CLI

```bash
npm install -g @arcium-hq/cli
# or
cargo install arcium-cli
```

### Step 2: Initialize Arcium Project

```bash
cd programs/arcium_poker
arcium init --template mpc-game
```

### Step 3: Define Confidential Instructions

Create `programs/arcium_poker/arcis/poker_mpc.rs`:

```rust
use arcis::prelude::*;

#[confidential]
pub fn shuffle_deck(
    deck: Enc<Shared, [u8; 52]>,
    entropy: Vec<Enc<Owner, [u8; 32]>>,
) -> Enc<Shared, [u8; 52]> {
    // MPC shuffle using Fisher-Yates with combined entropy
    let mut shuffled = deck.to_arcis();
    let combined_entropy = combine_entropy(entropy);
    
    for i in (1..52).rev() {
        let j = secure_random(combined_entropy, i);
        shuffled.swap(i, j);
    }
    
    Shared.from_arcis(shuffled)
}

#[confidential]
pub fn deal_card(
    deck: Enc<Shared, [u8; 52]>,
    index: u8,
    player: Pubkey,
) -> Enc<Owner, u8> {
    let deck_data = deck.to_arcis();
    let card = deck_data[index as usize];
    
    // Encrypt to specific player
    Owner(player).from_arcis(card)
}

#[confidential]
pub fn reveal_card(
    encrypted_card: Enc<Owner, u8>,
    requester: Pubkey,
) -> u8 {
    // Only owner can decrypt
    require!(encrypted_card.owner() == requester);
    encrypted_card.to_arcis()
}

#[confidential]
pub fn compare_hands(
    hand1: Enc<Owner, [u8; 2]>,
    hand2: Enc<Owner, [u8; 2]>,
    community: [u8; 5],
) -> u8 {
    // Evaluate hands in MPC without revealing cards
    let h1_value = evaluate_hand_mpc(hand1.to_arcis(), community);
    let h2_value = evaluate_hand_mpc(hand2.to_arcis(), community);
    
    if h1_value > h2_value { 1 }
    else if h2_value > h1_value { 2 }
    else { 0 }
}
```

### Step 4: Update Solana Program

Modify `programs/arcium_poker/src/arcium/mpc_shuffle.rs`:

```rust
use anchor_lang::prelude::*;
use arcium_sdk::prelude::*;

pub fn mpc_shuffle_deck(
    mxe_program: &Program<MxeProgram>,
    deck: &[u8; 52],
    player_entropy: &[[u8; 32]],
    mxe_ix_index: u8,
) -> Result<[u8; 52]> {
    // Create MXE instruction
    let ix_data = MxeInstructionData {
        ix_index: mxe_ix_index,
        encrypted_inputs: vec![
            encrypt_for_mxe(deck)?,
            encrypt_entropy_vec(player_entropy)?,
        ],
    };
    
    // Invoke MXE program
    let result = invoke_mxe(
        mxe_program,
        ix_data,
        &[/* required accounts */],
    )?;
    
    // Decrypt result
    decrypt_from_mxe(&result.encrypted_output)
}
```

### Step 5: Client-Side Integration

Create `app/src/arcium-client.ts`:

```typescript
import {
  RescueCipher,
  getArciumEnv,
  x25519,
  MxeClient,
} from "@arcium-hq/arcium-sdk";
import { Connection, PublicKey } from "@solana/web3.js";

export class PokerMpcClient {
  private cipher: RescueCipher;
  private mxeClient: MxeClient;
  private keypair: x25519.Keypair;

  constructor(connection: Connection, mxeProgramId: PublicKey) {
    this.keypair = x25519.generateKeypair();
    this.cipher = new RescueCipher();
    this.mxeClient = new MxeClient(connection, mxeProgramId);
  }

  // Encrypt player entropy for shuffle
  async encryptEntropy(entropy: Uint8Array): Promise<Uint8Array> {
    const nonce = crypto.getRandomValues(new Uint8Array(16));
    return this.cipher.encrypt(entropy, this.keypair.secretKey, nonce);
  }

  // Request shuffle from MXE
  async requestShuffle(
    gameId: PublicKey,
    playerEntropy: Uint8Array[]
  ): Promise<string> {
    const encryptedEntropy = await Promise.all(
      playerEntropy.map(e => this.encryptEntropy(e))
    );

    const tx = await this.mxeClient.invokeConfidential({
      programId: POKER_PROGRAM_ID,
      instructionIndex: 0, // shuffle_deck
      encryptedInputs: encryptedEntropy,
      accounts: [
        { pubkey: gameId, isSigner: false, isWritable: true },
        // ... other accounts
      ],
    });

    return tx;
  }

  // Decrypt dealt card
  async decryptCard(encryptedCard: Uint8Array): Promise<number> {
    return this.cipher.decrypt(
      encryptedCard,
      this.keypair.secretKey
    )[0];
  }

  // Request card reveal for showdown
  async revealCards(
    gameId: PublicKey,
    encryptedCards: Uint8Array[]
  ): Promise<number[]> {
    const tx = await this.mxeClient.invokeConfidential({
      programId: POKER_PROGRAM_ID,
      instructionIndex: 2, // reveal_card
      encryptedInputs: encryptedCards,
      accounts: [
        { pubkey: gameId, isSigner: false, isWritable: true },
      ],
    });

    // Wait for MXE callback
    const result = await this.mxeClient.waitForResult(tx);
    
    return result.outputs.map(o => this.decryptCard(o));
  }
}
```

### Step 6: Deployment Configuration

Update `Anchor.toml`:

```toml
[programs.devnet]
arcium_poker = "YOUR_PROGRAM_ID"
arcium_poker_mxe = "YOUR_MXE_PROGRAM_ID"

[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"

[arcium]
mxe_program_id = "YOUR_MXE_PROGRAM_ID"
cluster_id = "YOUR_CLUSTER_ID"
callback_url = "http://localhost:3000/callback"
```

### Step 7: Testing

```typescript
import { PokerMpcClient } from "./arcium-client";

describe("Poker MPC", () => {
  let mpcClient: PokerMpcClient;

  before(async () => {
    mpcClient = new PokerMpcClient(connection, mxeProgramId);
  });

  it("Shuffles deck with MPC", async () => {
    // Generate entropy from each player
    const entropy1 = crypto.getRandomValues(new Uint8Array(32));
    const entropy2 = crypto.getRandomValues(new Uint8Array(32));

    // Request shuffle
    const tx = await mpcClient.requestShuffle(
      gameId,
      [entropy1, entropy2]
    );

    // Verify shuffle completed
    const game = await program.account.game.fetch(gameId);
    assert(game.deckInitialized);
  });

  it("Deals encrypted cards", async () => {
    // Deal cards to player
    const cards = await mpcClient.dealCards(gameId, playerId, 2);
    
    // Cards should be encrypted
    assert(cards.length === 2);
    assert(cards[0] instanceof Uint8Array);
  });

  it("Reveals cards at showdown", async () => {
    // Reveal player's hole cards
    const revealed = await mpcClient.revealCards(
      gameId,
      encryptedHoleCards
    );

    // Should get actual card values
    assert(revealed.length === 2);
    assert(revealed[0] >= 0 && revealed[0] < 52);
  });
});
```

## Security Considerations

1. **Entropy Sources**: Each player must contribute entropy for shuffle
2. **Key Management**: Store x25519 keypairs securely
3. **Nonce Handling**: Never reuse nonces for encryption
4. **Callback Authentication**: Verify MXE callbacks are authentic
5. **Timeout Handling**: Handle MPC computation timeouts gracefully

## Performance Notes

- MPC computations take 2-5 seconds typically
- Shuffle operation: ~3-4 seconds
- Card dealing: ~1-2 seconds per card
- Showdown reveal: ~2-3 seconds

## Next Steps

1. Deploy MXE program to devnet
2. Set up Arcium cluster or join existing one
3. Configure callback server for async results
4. Test with multiple players
5. Monitor MXE execution logs
6. Optimize for mainnet deployment

## Resources

- [Arcium Docs](https://docs.arcium.com/)
- [TypeScript SDK](https://ts.arcium.com/)
- [Examples Repo](https://github.com/arcium-hq/examples)
- [Discord Support](https://discord.com/invite/arcium)
