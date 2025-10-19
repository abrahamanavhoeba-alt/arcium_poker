# 🎴 Arcium Poker - Quick Reference Card

## 📍 Essential Information

### **Program ID**
```
DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm
```

### **Network**
```
Devnet (https://api.devnet.solana.com)
```

### **Explorer**
```
https://explorer.solana.com/address/DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm?cluster=devnet
```

---

## 🔧 Frontend Setup (Copy-Paste Ready)

### **.env.local**
```bash
NEXT_PUBLIC_PROGRAM_ID=DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm
NEXT_PUBLIC_RPC_ENDPOINT=https://api.devnet.solana.com
NEXT_PUBLIC_NETWORK=devnet
```

### **Program Initialization**
```typescript
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import idl from "./idl/arcium_poker.json";

const PROGRAM_ID = new PublicKey("DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm");
const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const provider = new AnchorProvider(connection, wallet, {});
const program = new Program(idl, PROGRAM_ID, provider);
```

---

## 🎮 Common Operations

### **Create Game**
```typescript
const gameId = Date.now();
const [gamePda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("game"),
    wallet.publicKey.toBuffer(),
    new BN(gameId).toArrayLike(Buffer, "le", 8),
  ],
  program.programId
);

await program.methods
  .initializeGame(
    new BN(gameId),
    new BN(10),      // small blind
    new BN(20),      // big blind
    new BN(1000),    // min buy-in
    new BN(50000),   // max buy-in
    6                // max players
  )
  .accounts({ authority: wallet.publicKey })
  .rpc();
```

### **Join Game**
```typescript
await program.methods
  .joinGame(new BN(10000))  // buy-in amount
  .accounts({
    game: gamePda,
    player: wallet.publicKey,
  })
  .rpc();
```

### **Place Bet**
```typescript
await program.methods
  .bet(new BN(100))
  .accounts({
    game: gamePda,
    player: wallet.publicKey,
  })
  .rpc();
```

### **Read Game State**
```typescript
const gameAccount = await program.account.game.fetch(gamePda);
console.log("Pot:", gameAccount.pot.toNumber());
console.log("Stage:", Object.keys(gameAccount.stage)[0]);
console.log("Players:", gameAccount.players.length);
```

---

## 📦 Required Files

Copy from smart contract to frontend:

```bash
target/idl/arcium_poker.json     → src/idl/arcium_poker.json
target/types/arcium_poker.ts     → src/types/arcium_poker.ts
```

---

## 🧪 Quick Test

```typescript
// Test if program is deployed
const connection = new Connection("https://api.devnet.solana.com");
const programId = new PublicKey("DmthLucwUx2iM7VoFUv14PHfVqfqGxHKLMVXzUb8vvMm");
const info = await connection.getAccountInfo(programId);
console.log(info ? "✅ Live" : "❌ Not found");
```

---

## 📋 Available Instructions

| Instruction | Description |
|-------------|-------------|
| `initialize_game` | Create new game |
| `join_game` | Join with buy-in |
| `start_game` | Start game (shuffle) |
| `bet` | Place bet |
| `call` | Call current bet |
| `fold` | Fold hand |
| `check` | Check (no bet) |
| `raise` | Raise bet |
| `all_in` | Go all-in |
| `advance_stage` | Next stage |
| `execute_showdown` | Reveal & distribute |
| `end_game` | End game |

---

## 🔗 Useful Links

- **Solana Explorer**: https://explorer.solana.com
- **Solana Faucet**: https://faucet.solana.com
- **Anchor Docs**: https://www.anchor-lang.com
- **Solana Docs**: https://docs.solana.com

---

## 💡 Tips

1. **Always use Devnet** - This is a test deployment
2. **Get test SOL** - Use Solana faucet for devnet SOL
3. **Check wallet network** - Ensure wallet is on Devnet
4. **Use confirmed commitment** - For reliable transaction status
5. **Handle errors** - Network issues are common on devnet

---

## 🆘 Troubleshooting

### Program not found
- Check you're on devnet
- Verify program ID is correct
- Try different RPC endpoint

### Transaction failed
- Check wallet has SOL
- Verify account derivation
- Check instruction parameters

### Simulation failed
- Read error message carefully
- Check account permissions
- Verify signer requirements

---

**Quick Start**: Copy `.env.local`, copy IDL/types, initialize program, start coding! 🚀
