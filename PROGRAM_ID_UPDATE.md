# Program ID Update - Oct 24, 2025

## Issue
Cross-program invocation error: "writable privilege escalated"

## Root Cause
In `/programs/arcium_poker/src/arcium/integration.rs`, the `comp_def` account was marked as **readonly** in the CPI call, but it was declared as **writable** (`mut`) in the `StartGame` instruction struct.

## Fix Applied

### 1. Fixed CPI Account Permission (Line 122)
**File:** `/programs/arcium_poker/src/arcium/integration.rs`

**Before:**
```rust
anchor_lang::solana_program::instruction::AccountMeta::new_readonly(*comp_def.key, false),
```

**After:**
```rust
anchor_lang::solana_program::instruction::AccountMeta::new(*comp_def.key, false), // Changed to writable
```

### 2. Updated Program ID (New Deployment)

**Old Program ID:** `FHzVm4eu5ZuuzX3W4YRD8rS6XZVrdXubrJnYTqgBYZu2`  
**New Program ID:** `AshR4SHHiPJAKFbPjeeCkH4TNw82bbrJwmzbP4dThQKQ`

## Files Updated

### Backend (Arcium Poker Contract)
1. ✅ `/programs/arcium_poker/src/lib.rs` - Updated `declare_id!` macro
2. ✅ `/programs/arcium_poker/src/arcium/integration.rs` - Fixed CPI account permissions
3. ✅ `/Anchor.toml` - Updated program IDs for localnet and devnet
4. ✅ `/target/idl/arcium_poker.json` - Regenerated with new program ID

### Frontend (Arcium Poker Frontend)
1. ✅ `/src/lib/shared/constants.ts` - Updated `PROGRAM_ID` constant
2. ✅ `/src/hooks/useStartGame.ts` - Updated `MXE_PROGRAM_ID`
3. ✅ `/src/arcium_poker.json` - Updated IDL with new program address

## Deployment

**Network:** Devnet  
**Program ID:** `AshR4SHHiPJAKFbPjeeCkH4TNw82bbrJwmzbP4dThQKQ`  
**Deployment Signature:** `3HCfSiX1Yfy9P7u2ufY5rFWVH1HM4tTYfDQhHn6oqMs7oAyUb5qs5T6pKPuMGwHA7TMxv5ucmwjF2kz22hJCYPAw`

## Next Steps

1. ✅ Test the `start_game` function with the fixed contract
2. ✅ Verify all MXE accounts are properly passed as writable
3. ⏳ Initialize a new game with the updated program
4. ⏳ Test complete game flow end-to-end

## Notes

- The fix ensures that when making CPI calls to the MXE program, all accounts declared as writable in the parent instruction are also passed as writable in the CPI call
- This prevents Solana's runtime from rejecting the transaction due to privilege escalation
- The program was redeployed to ensure all changes are live on devnet
