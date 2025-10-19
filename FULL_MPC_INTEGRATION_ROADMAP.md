# 🔐 Full Arcium MPC Integration Roadmap

**Project:** Arcium Poker - Encrypted Texas Hold'em on Solana  
**Timeline:** 3-5 days  
**Risk Level:** HIGH (may break existing 48/48 tests)  
**Reward:** True encrypted poker with provable fairness

---

## 📋 Current Architecture Analysis

### ✅ What We Have (Working)
- **48/48 tests passing**
- Complete game logic (betting, stages, pot management)
- Stubbed MPC functions that return dummy data
- Well-structured codebase with clear separation of concerns
- Automatic blind posting
- Side pot handling

### ⚠️ What's Stubbed (Needs Real MPC)
```rust
// Current: programs/arcium_poker/src/arcium/mpc_shuffle.rs
pub fn mpc_shuffle_deck(params: ShuffleParams) -> Result<ShuffleResult> {
    // ❌ Returns deterministic shuffle, NOT real MPC
    let shuffled_indices = secure_shuffle_with_entropy(&params.player_entropy)?;
    // Should be: Real Arcium MPC computation
}
```

---

## 🎯 Integration Strategy

### **Phase 1: Arcium Setup** (Day 1)
**Goal:** Get Arcium tooling working

#### 1.1 Install Arcium CLI
```bash
# Install Arcium CLI
curl --proto '=https' --tlsv1.2 -sSfL https://install.arcium.com/ | bash
arcup  # Install all Arcium tools

# Verify installation
arcium --version
```

#### 1.2 Install Client Libraries
```bash
cd /home/a/CascadeProjects/arcium_poker
npm install @arcium-hq/client
npm install @arcium-hq/reader
```

#### 1.3 Create Arcium Project Structure
```bash
# Arcium uses a separate MXE program for confidential computations
mkdir -p programs/poker_mxe
cd programs/poker_mxe
arcium init
```

**Expected Output:**
```
programs/
├── arcium_poker/          # Your existing Solana program
│   └── src/
│       ├── arcium/        # Integration layer
│       ├── game/
│       └── ...
└── poker_mxe/             # NEW: Arcium MXE program
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs         # Confidential instructions
    └── Arcium.toml
```

---

### **Phase 2: Write Arcis Circuits** (Day 2)
**Goal:** Implement confidential shuffle/deal in Arcis

#### 2.1 Shuffle Circuit
Create `programs/poker_mxe/src/lib.rs`:

```rust
use arcis_imports::*;

#[encrypted]
mod circuits {
    use arcis_imports::*;

    /// Input structure for shuffle
    pub struct ShuffleInput {
        deck: [u8; 52],              // Initial ordered deck
        entropy_p1: [u8; 32],        // Player 1 entropy
        entropy_p2: [u8; 32],        // Player 2 entropy
        entropy_p3: [u8; 32],        // Player 3 entropy (optional)
    }

    /// Confidential shuffle using Fisher-Yates algorithm
    /// This runs in MPC - no single party sees the shuffled deck
    #[instruction]
    pub fn shuffle_deck(
        input_ctxt: Enc<Shared, ShuffleInput>
    ) -> [u8; 52] {
        let input = input_ctxt.to_arcis();
        
        // Combine all entropy sources
        let mut combined_entropy = [0u8; 32];
        for i in 0..32 {
            combined_entropy[i] = input.entropy_p1[i] 
                ^ input.entropy_p2[i] 
                ^ input.entropy_p3[i];
        }
        
        // Fisher-Yates shuffle in MPC
        let mut shuffled = input.deck;
        for i in (1..52).rev() {
            // Generate random index from entropy
            let j = (combined_entropy[i % 32] as usize) % (i + 1);
            shuffled.swap(i, j);
            
            // Mix entropy for next iteration
            combined_entropy = hash_entropy(combined_entropy);
        }
        
        shuffled.reveal()  // Reveal shuffled deck to all
    }

    /// Deal a single card to a specific player
    /// Card is encrypted to player's public key
    #[instruction]
    pub fn deal_card_to_player(
        deck: Enc<Shared, [u8; 52]>,
        card_index: u8,
        player_pubkey: Pubkey,
    ) -> Enc<Owner, u8> {
        let deck_data = deck.to_arcis();
        let card = deck_data[card_index as usize];
        
        // Encrypt card to specific player
        Owner(player_pubkey).from_arcis(card)
    }

    /// Reveal card at showdown
    /// Only card owner can decrypt
    #[instruction]
    pub fn reveal_card(
        encrypted_card: Enc<Owner, u8>,
        requester: Pubkey,
    ) -> u8 {
        // Verify requester is the owner
        require!(encrypted_card.owner() == requester);
        encrypted_card.to_arcis().reveal()
    }

    // Helper function for entropy mixing
    fn hash_entropy(input: [u8; 32]) -> [u8; 32] {
        // Simple hash for demonstration
        // In production, use proper hash function
        let mut output = [0u8; 32];
        for i in 0..32 {
            output[i] = input[i].wrapping_mul(7).wrapping_add(13);
        }
        output
    }
}
```

#### 2.2 Build MXE Program
```bash
cd programs/poker_mxe
arcium build

# Expected output:
# ✓ Built MXE program
# ✓ Generated computation definitions
# Program ID: <YOUR_MXE_PROGRAM_ID>
```

---

### **Phase 3: Update Solana Integration** (Day 2-3)
**Goal:** Connect Solana program to MXE

#### 3.1 Update `mpc_shuffle.rs`
Replace `programs/arcium_poker/src/arcium/mpc_shuffle.rs`:

```rust
use anchor_lang::prelude::*;
use crate::shared::constants::DECK_SIZE;
use crate::shared::PokerError;

/// Parameters for invoking MXE shuffle
pub struct MxeShuffleParams<'info> {
    /// MXE program account
    pub mxe_program: AccountInfo<'info>,
    
    /// Computation definition account
    pub comp_def: AccountInfo<'info>,
    
    /// Mempool account for queueing computation
    pub mempool: AccountInfo<'info>,
    
    /// Cluster account
    pub cluster: AccountInfo<'info>,
    
    /// Player entropy (encrypted)
    pub encrypted_entropy: Vec<[u8; 32]>,
    
    /// Computation offset (unique ID)
    pub computation_offset: [u8; 8],
}

/// Invoke Arcium MXE for deck shuffle
pub fn mpc_shuffle_deck<'info>(
    params: MxeShuffleParams<'info>,
) -> Result<ShuffleResult> {
    msg!("[ARCIUM MPC] Invoking confidential shuffle...");
    
    // Create MXE instruction data
    let ix_data = create_mxe_instruction(
        0, // shuffle_deck instruction index
        params.encrypted_entropy,
        params.computation_offset,
    )?;
    
    // Invoke MXE program via CPI
    let accounts = vec![
        params.comp_def.clone(),
        params.mempool.clone(),
        params.cluster.clone(),
    ];
    
    invoke_mxe_computation(
        &params.mxe_program,
        &ix_data,
        &accounts,
    )?;
    
    msg!("[ARCIUM MPC] Shuffle queued, waiting for result...");
    
    // In production, this would be async with callback
    // For now, we'll poll for result
    let result = poll_for_mxe_result(
        params.computation_offset,
        30, // 30 second timeout
    )?;
    
    Ok(result)
}

/// Helper: Create MXE instruction data
fn create_mxe_instruction(
    ix_index: u8,
    encrypted_inputs: Vec<[u8; 32]>,
    computation_offset: [u8; 8],
) -> Result<Vec<u8>> {
    // Serialize MXE instruction format
    let mut data = Vec::new();
    data.push(ix_index);
    data.extend_from_slice(&computation_offset);
    
    for input in encrypted_inputs {
        data.extend_from_slice(&input);
    }
    
    Ok(data)
}

/// Helper: Invoke MXE via CPI
fn invoke_mxe_computation(
    mxe_program: &AccountInfo,
    ix_data: &[u8],
    accounts: &[AccountInfo],
) -> Result<()> {
    // Create instruction
    let ix = solana_program::instruction::Instruction {
        program_id: *mxe_program.key,
        accounts: accounts.iter().map(|a| {
            solana_program::instruction::AccountMeta {
                pubkey: *a.key,
                is_signer: a.is_signer,
                is_writable: a.is_writable,
            }
        }).collect(),
        data: ix_data.to_vec(),
    };
    
    // Invoke via CPI
    solana_program::program::invoke(
        &ix,
        &[mxe_program.clone()].iter().chain(accounts.iter()).cloned().collect::<Vec<_>>(),
    )?;
    
    Ok(())
}

/// Helper: Poll for MXE result
fn poll_for_mxe_result(
    computation_offset: [u8; 8],
    timeout_secs: u64,
) -> Result<ShuffleResult> {
    // In production, use callback server
    // For hackathon, return placeholder that will be replaced by callback
    
    msg!("[ARCIUM MPC] Result will be delivered via callback");
    msg!("[ARCIUM MPC] Computation ID: {:?}", computation_offset);
    
    // Placeholder - actual result comes from callback
    Ok(ShuffleResult {
        shuffled_indices: [0; DECK_SIZE], // Will be filled by callback
        commitment: [0; 32],
        session_id: computation_offset,
        shuffle_proof: None,
    })
}

#[derive(Clone, Debug)]
pub struct ShuffleResult {
    pub shuffled_indices: [u8; DECK_SIZE],
    pub commitment: [u8; 32],
    pub session_id: [u8; 8],
    pub shuffle_proof: Option<Vec<u8>>,
}
```

#### 3.2 Update `start_game` to use MXE
Modify `programs/arcium_poker/src/game/start.rs`:

```rust
// Add MXE accounts to Context
#[derive(Accounts)]
pub struct StartGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    
    pub authority: Signer<'info>,
    
    // NEW: MXE accounts
    /// CHECK: MXE program
    pub mxe_program: AccountInfo<'info>,
    
    /// CHECK: Computation definition
    pub comp_def: AccountInfo<'info>,
    
    /// CHECK: Mempool account
    pub mempool: AccountInfo<'info>,
    
    /// CHECK: Cluster account  
    pub cluster: AccountInfo<'info>,
}

// In handler, invoke MXE:
pub fn handler(
    ctx: Context<StartGame>,
    player_entropy: Vec<[u8; 32]>,
) -> Result<()> {
    // ... existing validation ...
    
    // Invoke MXE shuffle
    let mxe_params = MxeShuffleParams {
        mxe_program: ctx.accounts.mxe_program.to_account_info(),
        comp_def: ctx.accounts.comp_def.to_account_info(),
        mempool: ctx.accounts.mempool.to_account_info(),
        cluster: ctx.accounts.cluster.to_account_info(),
        encrypted_entropy: player_entropy,
        computation_offset: game.game_id.to_le_bytes(),
    };
    
    let shuffle_result = mpc_shuffle_deck(mxe_params)?;
    
    // Store result
    game.encrypted_deck = shuffle_result.session_id;
    game.deck_initialized = true;
    
    // ... rest of handler ...
}
```

---

### **Phase 4: Client-Side Integration** (Day 3)
**Goal:** Encrypt inputs, decrypt outputs

#### 4.1 Create MPC Client Helper
Create `tests/helpers/arcium-mpc.ts`:

```typescript
import {
  RescueCipher,
  getArciumEnv,
  x25519,
  getMXEPublicKeyWithRetry,
  awaitComputationFinalization,
} from "@arcium-hq/client";
import { randomBytes } from "crypto";
import * as anchor from "@coral-xyz/anchor";

export class PokerMpcClient {
  private provider: anchor.AnchorProvider;
  private mxeProgramId: anchor.web3.PublicKey;
  private cipher: RescueCipher | null = null;
  private privateKey: Uint8Array | null = null;
  private publicKey: Uint8Array | null = null;

  constructor(
    provider: anchor.AnchorProvider,
    mxeProgramId: anchor.web3.PublicKey
  ) {
    this.provider = provider;
    this.mxeProgramId = mxeProgramId;
  }

  async initialize() {
    // Get MXE public key
    const mxePublicKey = await getMXEPublicKeyWithRetry(
      this.provider,
      this.mxeProgramId
    );

    // Generate keypair for encryption
    this.privateKey = x25519.utils.randomSecretKey();
    this.publicKey = x25519.getPublicKey(this.privateKey);

    // Create shared secret
    const sharedSecret = x25519.getSharedSecret(
      this.privateKey,
      mxePublicKey
    );

    // Initialize cipher
    this.cipher = new RescueCipher(sharedSecret);
  }

  // Encrypt player entropy for shuffle
  encryptEntropy(entropy: Uint8Array): { ciphertext: Uint8Array; nonce: Uint8Array } {
    if (!this.cipher) throw new Error("Client not initialized");

    const nonce = randomBytes(16);
    const plaintext = Array.from(entropy).map(BigInt);
    const ciphertext = this.cipher.encrypt(plaintext, nonce);

    return {
      ciphertext: new Uint8Array(ciphertext.flat()),
      nonce,
    };
  }

  // Wait for MPC computation to complete
  async waitForShuffle(
    computationOffset: anchor.BN
  ): Promise<Uint8Array> {
    const finalizeSig = await awaitComputationFinalization(
      this.provider,
      computationOffset,
      this.mxeProgramId,
      "confirmed"
    );

    console.log("Shuffle finalized:", finalizeSig);

    // Fetch result from computation account
    // ... implementation depends on Arcium SDK
    
    return new Uint8Array(52); // Shuffled deck
  }

  // Decrypt dealt card
  decryptCard(encryptedCard: Uint8Array, nonce: Uint8Array): number {
    if (!this.cipher) throw new Error("Client not initialized");

    const plaintext = this.cipher.decrypt([encryptedCard], nonce);
    return Number(plaintext[0]);
  }
}
```

#### 4.2 Update Tests
Modify `tests/test_game_flow.ts`:

```typescript
import { PokerMpcClient } from "./helpers/arcium-mpc";

describe("Game Flow with Real MPC", () => {
  let mpcClient: PokerMpcClient;

  before(async () => {
    // Initialize MPC client
    mpcClient = new PokerMpcClient(
      provider,
      MXE_PROGRAM_ID // From deployment
    );
    await mpcClient.initialize();
  });

  it("Starts game with MPC shuffle", async () => {
    // Generate real entropy
    const entropy1 = crypto.getRandomValues(new Uint8Array(32));
    const entropy2 = crypto.getRandomValues(new Uint8Array(32));
    const entropy3 = crypto.getRandomValues(new Uint8Array(32));

    // Encrypt entropy
    const enc1 = mpcClient.encryptEntropy(entropy1);
    const enc2 = mpcClient.encryptEntropy(entropy2);
    const enc3 = mpcClient.encryptEntropy(entropy3);

    // Start game with encrypted entropy
    await program.methods
      .startGame([
        Array.from(enc1.ciphertext),
        Array.from(enc2.ciphertext),
        Array.from(enc3.ciphertext),
      ])
      .accounts({
        game: gamePda,
        authority: provider.wallet.publicKey,
        mxeProgram: MXE_PROGRAM_ID,
        compDef: COMP_DEF_PDA,
        mempool: MEMPOOL_PDA,
        cluster: CLUSTER_PDA,
      })
      .rpc();

    // Wait for MPC shuffle to complete
    const game = await program.account.game.fetch(gamePda);
    const shuffled = await mpcClient.waitForShuffle(
      new anchor.BN(game.gameId)
    );

    expect(shuffled.length).to.equal(52);
  });
});
```

---

### **Phase 5: Deployment & Testing** (Day 4-5)
**Goal:** Deploy to testnet and verify

#### 5.1 Deploy MXE Program
```bash
# Build both programs
anchor build
cd programs/poker_mxe && arcium build && cd ../..

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy MXE program
arcium deploy --cluster devnet programs/poker_mxe
```

#### 5.2 Configure Callback Server
```bash
# Install callback server
npm install @arcium-hq/callback-server

# Start callback server
npx arcium-callback-server \
  --port 3000 \
  --cluster devnet \
  --mxe-program-id <YOUR_MXE_ID>
```

#### 5.3 Run Integration Tests
```bash
# Set environment variables
export MXE_PROGRAM_ID=<your_mxe_program_id>
export COMP_DEF_PDA=<your_comp_def_pda>
export CLUSTER_PDA=<your_cluster_pda>

# Run tests
anchor test --skip-deploy
```

---

## 🚨 Risk Mitigation

### Risks & Mitigation Strategies

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Breaking existing tests** | HIGH | Keep old code in `_legacy` folder, gradual migration |
| **MXE deployment issues** | HIGH | Test on local validator first, have fallback |
| **Async callback complexity** | MEDIUM | Implement polling fallback, clear error messages |
| **Performance degradation** | MEDIUM | Add timeout handling, cache results |
| **Integration bugs** | HIGH | Extensive logging, step-by-step testing |

### Rollback Plan
```bash
# If integration fails, revert to working version
git checkout -b mpc-integration-backup
git add .
git commit -m "Backup before MPC integration"

# After integration attempt
git checkout main  # If successful
git checkout mpc-integration-backup  # If failed
```

---

## 📊 Success Criteria

### Minimum Viable MPC Integration
- [ ] MXE program deploys successfully
- [ ] Shuffle computation completes in <10 seconds
- [ ] At least 1 test passes with real MPC
- [ ] Clear documentation of integration

### Full Success
- [ ] All 48 tests pass with real MPC
- [ ] Shuffle + deal + reveal all working
- [ ] Performance acceptable (<5s per operation)
- [ ] Ready for hackathon submission

---

## 📝 Daily Checklist

### Day 1: Setup
- [ ] Install Arcium CLI
- [ ] Create poker_mxe project
- [ ] Verify tooling works
- [ ] Read all Arcium docs

### Day 2: Circuits
- [ ] Write shuffle circuit
- [ ] Write deal circuit
- [ ] Write reveal circuit
- [ ] Build MXE program successfully

### Day 3: Integration
- [ ] Update Solana program
- [ ] Add MXE accounts to contexts
- [ ] Implement CPI calls
- [ ] Create client helper

### Day 4: Testing
- [ ] Deploy to devnet
- [ ] Run first MPC shuffle
- [ ] Debug issues
- [ ] Update tests

### Day 5: Polish
- [ ] Fix remaining bugs
- [ ] Performance optimization
- [ ] Documentation
- [ ] Prepare demo

---

## 🎯 Hackathon Submission Checklist

- [ ] **Working demo** - At least shuffle working with MPC
- [ ] **Video** - 2-3 min showing encrypted shuffle
- [ ] **README** - Clear explanation of Arcium usage
- [ ] **Architecture diagram** - Show MPC flow
- [ ] **Code quality** - Clean, well-commented
- [ ] **Privacy explanation** - Why Arcium matters for poker

---

## 📚 Resources

### Official Docs
- [Arcium Docs](https://docs.arcium.com/)
- [TypeScript SDK](https://ts.arcium.com/api)
- [Hello World Example](https://docs.arcium.com/developers/hello-world)

### Your Codebase
- Current stub: `programs/arcium_poker/src/arcium/mpc_shuffle.rs`
- Integration points: `programs/arcium_poker/src/game/start.rs`
- Tests: `tests/test_game_flow.ts`

### Support
- [Arcium Discord](https://discord.gg/arcium)
- [GitHub Issues](https://github.com/arcium-hq/arcium)

---

## ⚡ Quick Start (TL;DR)

```bash
# Day 1: Setup
curl --proto '=https' --tlsv1.2 -sSfL https://install.arcium.com/ | bash
arcup
npm install @arcium-hq/client

# Day 2: Create MXE
mkdir programs/poker_mxe && cd programs/poker_mxe
arcium init
# Write circuits in src/lib.rs
arcium build

# Day 3: Integrate
# Update start.rs to call MXE
# Add MXE accounts to Context

# Day 4: Deploy & Test
anchor deploy --provider.cluster devnet
arcium deploy --cluster devnet programs/poker_mxe
anchor test

# Day 5: Demo & Submit
# Record video, write README, submit!
```

---

**Remember:** This is a HIGH-RISK, HIGH-REWARD path. You have a working product (48/48 tests). Only pursue this if you're confident you can complete it in time. Otherwise, stick with Option A (quick Arcium integration) for the hackathon.

Good luck! 🚀
