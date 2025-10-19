# 🚀 Deployment Information

## Contract Deployment Details

**Deployment Date**: October 19, 2025  
**Network**: Solana Devnet  
**Status**: ✅ Successfully Deployed

---

## Contract Information

| Property | Value |
|----------|-------|
| **Program ID** | `DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm` |
| **Network** | Devnet |
| **RPC Endpoint** | `https://api.devnet.solana.com` |
| **Program Size** | 426 KB (435,208 bytes) |
| **Deployed Slot** | 415,670,316 |
| **Deployer Wallet** | `DCZZHgdED5uGgYSG6sTViAwJtDZjn3Fij9u8TAQZG1jC` |
| **Program Data Address** | `5e9hsWzx3SmcRfToPyJZcEWtSfQE54hs9eJLhYTaWmgg` |
| **Upgrade Authority** | `DCZZHgdED5uGgYSG6sTViAwJtDZjn3Fij9u8TAQZG1jC` |

---

## Explorer Links

- **Solana Explorer**: [View Program](https://explorer.solana.com/address/DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm?cluster=devnet)
- **Solscan**: [View on Solscan](https://solscan.io/account/DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm?cluster=devnet)

---

## Frontend Integration

### Environment Variables

Create a `.env.local` file in your frontend project:

```bash
NEXT_PUBLIC_PROGRAM_ID=DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm
NEXT_PUBLIC_RPC_ENDPOINT=https://api.devnet.solana.com
NEXT_PUBLIC_NETWORK=devnet
```

### Required Files

Copy these files from the smart contract to your frontend:

```bash
# From smart contract project
cp target/idl/arcium_poker.json <frontend>/src/idl/
cp target/types/arcium_poker.ts <frontend>/src/types/
```

### TypeScript Setup

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import idl from "./idl/arcium_poker.json";
import { ArciumPoker } from "./types/arcium_poker";

// Configuration
const PROGRAM_ID = new PublicKey(
  process.env.NEXT_PUBLIC_PROGRAM_ID || 
  "DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm"
);

const RPC_ENDPOINT = 
  process.env.NEXT_PUBLIC_RPC_ENDPOINT || 
  "https://api.devnet.solana.com";

// Initialize program
export function getProgram(wallet: any): Program<ArciumPoker> {
  const connection = new Connection(RPC_ENDPOINT, "confirmed");
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  
  return new Program<ArciumPoker>(
    idl as any,
    PROGRAM_ID,
    provider
  );
}
```

---

## Available Instructions

The deployed program supports the following instructions:

### Game Management
- `initialize_game` - Create a new poker game
- `start_game` - Start the game (shuffle deck via MPC)
- `end_game` - End the game
- `advance_stage` - Move to next stage (Flop → Turn → River → Showdown)

### Player Actions
- `join_game` - Join a game with buy-in
- `leave_game` - Leave the game
- `bet` - Place a bet
- `call` - Call current bet
- `fold` - Fold hand
- `check` - Check (no bet)
- `raise` - Raise the bet
- `all_in` - Go all-in

### Game Flow
- `deal_hole_cards` - Deal private cards to players
- `deal_community_cards` - Deal flop/turn/river
- `execute_showdown` - Reveal cards and determine winner
- `distribute_pot` - Distribute winnings

---

## Testing the Deployment

### Quick Test Script

```typescript
import { Connection, PublicKey } from "@solana/web3.js";

async function testDeployment() {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const programId = new PublicKey("DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm");
  
  // Check if program exists
  const accountInfo = await connection.getAccountInfo(programId);
  
  if (accountInfo) {
    console.log("✅ Program deployed successfully!");
    console.log("Program size:", accountInfo.data.length, "bytes");
    console.log("Owner:", accountInfo.owner.toString());
  } else {
    console.log("❌ Program not found");
  }
}

testDeployment();
```

### Using Anchor CLI

```bash
# Set cluster to devnet
solana config set --url devnet

# Check program info
solana program show DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm

# Run tests against deployed program
anchor test --skip-build --skip-deploy
```

---

## Wallet Configuration

To interact with the deployed program, users need:

1. **Solana Wallet** (Phantom, Solflare, etc.)
2. **Devnet SOL** - Get from [Solana Faucet](https://faucet.solana.com/)
3. **Network set to Devnet** in wallet settings

---

## Security Notes

- ✅ Program is deployed with upgrade authority
- ✅ Upgrade authority: `DCZZHgdED5uGgYSG6sTViAwJtDZjn3Fij9u8TAQZG1jC`
- ⚠️ This is a devnet deployment for testing/development
- ⚠️ Do not use real funds - devnet only
- ⚠️ Program can be upgraded by the authority

---

## Next Steps

### For Frontend Development
1. ✅ Copy IDL and type files to frontend
2. ✅ Set up environment variables
3. ✅ Initialize Anchor program connection
4. ⏳ Build service layer for business logic
5. ⏳ Create React components
6. ⏳ Test end-to-end integration

### For Production Deployment
1. ⏳ Audit smart contract code
2. ⏳ Deploy MXE circuits for real MPC
3. ⏳ Deploy to mainnet-beta
4. ⏳ Transfer or revoke upgrade authority
5. ⏳ Set up monitoring and alerts

---

## Support & Resources

- **Documentation**: See [README.md](./README.md)
- **Integration Guide**: See [PHASE_3_COMPLETE.md](./PHASE_3_COMPLETE.md)
- **Test Examples**: See [tests/test_mxe_integration.ts](./tests/test_mxe_integration.ts)
- **Solana Docs**: https://docs.solana.com
- **Anchor Docs**: https://www.anchor-lang.com

---

## Changelog

### v1.0.0 - October 19, 2025
- ✅ Initial deployment to Devnet
- ✅ All 48 tests passing
- ✅ Dual-mode MPC integration (mock/real)
- ✅ Complete poker game logic
- ✅ IDL and types generated

---

**Deployed by**: [@ANAVHEOBA](https://github.com/ANAVHEOBA)  
**Project**: Arcium Poker - Encrypted Texas Hold'em  
**Hackathon**: Arcium's <encrypted> Side Track
