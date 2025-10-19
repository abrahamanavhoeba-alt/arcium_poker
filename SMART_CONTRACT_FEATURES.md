# Smart Contract Features - Arcium Poker

## Core Smart Contract Architecture

### 1. Game State Management

#### Game Initialization
- **Create Game Room**
  - Initialize game with configurable parameters (buy-in, blinds, max players)
  - Set up encrypted game state using Arcium MPC
  - Generate unique game ID
  - Define small blind / big blind amounts
  - Set minimum/maximum buy-in amounts
  - Support 4-6 players per table

#### Player Management
- **Join Game**
  - Validate player has sufficient balance
  - Lock player's buy-in amount in escrow
  - Assign player seat position
  - Initialize player's encrypted hand state
  
- **Leave Game**
  - Return remaining chips to player
  - Handle mid-game exits (fold and leave)
  - Redistribute seats if needed

### 2. Arcium MPC Integration (CRITICAL)

#### Encrypted Card Deck
- **Deck Initialization**
  - Generate 52-card deck using Arcium's MPC
  - Encrypt entire deck state
  - No single party can see deck order
  
#### Secure Shuffling
- **MPC-based Shuffle**
  - Multi-party computation shuffle algorithm
  - Cryptographically secure randomness
  - Verifiable shuffle proof
  - Each player contributes to shuffle entropy

#### Private Card Dealing
- **Encrypted Hand Distribution**
  - Deal 2 hole cards per player (encrypted)
  - Only card owner can decrypt their hand
  - Cards remain encrypted until showdown
  - Support for partial reveals (showdown logic)

#### Community Cards
- **Encrypted Flop/Turn/River**
  - Burn cards (encrypted, never revealed)
  - Reveal community cards at appropriate stages
  - Maintain encryption of undealt cards

### 3. Betting Mechanics

#### Betting Rounds
- **Pre-flop Betting**
  - Small blind / big blind posting
  - Action rotation (clockwise from dealer)
  - Support: fold, call, raise, all-in
  
- **Post-flop Betting** (Flop, Turn, River)
  - Check/bet/raise/fold actions
  - Minimum raise validation (2x previous bet)
  - Maximum raise caps (pot limit / no limit)

#### Pot Management
- **Main Pot & Side Pots**
  - Calculate main pot
  - Create side pots for all-in scenarios
  - Track eligible players per pot
  - Accurate pot distribution

#### Bet Validation
- **Action Validation**
  - Verify player has sufficient chips
  - Validate bet amounts (min/max)
  - Enforce betting order
  - Timeout handling for inactive players

### 4. Game Flow Control

#### Round Progression
- **Automatic Stage Transitions**
  - Pre-flop → Flop → Turn → River → Showdown
  - Trigger community card reveals
  - Reset betting state per round
  - Track active players

#### Turn Management
- **Action Timer**
  - 30-60 second turn timer per player
  - Auto-fold on timeout
  - Skip eliminated players
  - Handle disconnections gracefully

#### Dealer Button Rotation
- **Position Management**
  - Rotate dealer button each hand
  - Update small/big blind positions
  - Track button position in game state

### 5. Showdown & Hand Evaluation

#### Encrypted Showdown
- **Selective Card Reveal**
  - Players reveal hands only if required
  - Muck option (fold without showing)
  - Decrypt hands using Arcium MPC
  - Verify hand authenticity

#### Hand Ranking
- **Poker Hand Evaluation**
  - Royal Flush → High Card ranking
  - Compare hands cryptographically
  - Handle split pots (ties)
  - Kicker comparison logic

#### Winner Determination
- **Pot Distribution**
  - Identify winner(s) per pot
  - Calculate exact chip distribution
  - Handle side pot winners
  - Transfer winnings to player accounts

### 6. Token Integration (SPL Token)

#### USDC/SOL Betting
- **Token Escrow**
  - Lock tokens when joining game
  - Hold tokens in program-owned account
  - Release tokens on game completion
  
- **Chip Conversion**
  - Convert USDC/SOL to in-game chips
  - 1:1 ratio or configurable rate
  - Track chip balances per player

- **Withdrawals**
  - Cash out chips to USDC/SOL
  - Instant settlement after hand completion
  - Fee deduction (rake) if applicable

### 7. Security & Anti-Cheat

#### Encryption Guarantees
- **Zero-Knowledge Proofs**
  - Prove hand validity without revealing cards
  - Verify shuffle integrity
  - Prevent card manipulation

#### Collusion Prevention
- **Game Integrity**
  - No player can see other hands pre-showdown
  - Shuffle is verifiably random
  - All actions are on-chain and auditable
  - Timeout mechanisms prevent stalling

#### State Validation
- **Invariant Checks**
  - Total chips = sum of player stacks + pots
  - Deck integrity (52 unique cards)
  - Valid game state transitions only

### 8. Advanced Features (If Time Permits)

#### Tournament Mode
- **Multi-table Tournaments**
  - Increasing blinds over time
  - Player elimination tracking
  - Final table consolidation

#### Rake System
- **House Fee**
  - Configurable rake percentage (2-5%)
  - Rake cap per hand
  - Accumulated rake tracking

#### Statistics Tracking
- **Player Stats**
  - Hands played
  - Win rate
  - Total winnings
  - Biggest pot won

## Technical Implementation Priority

### Phase 1: MVP (Days 1-5)
1. ✅ Game initialization & player join/leave
2. ✅ Arcium MPC deck generation & shuffle
3. ✅ Encrypted card dealing (hole cards + community)
4. ✅ Basic betting mechanics (fold, call, raise)
5. ✅ Simple pot management (main pot only)

### Phase 2: Core Gameplay (Days 6-9)
6. ✅ Full betting round implementation
7. ✅ Showdown with encrypted reveal
8. ✅ Hand evaluation & winner determination
9. ✅ USDC/SOL token integration
10. ✅ Turn timer & game flow automation

### Phase 3: Polish (Days 10-12)
11. ✅ Side pot calculations
12. ✅ Anti-cheat validations
13. ✅ Error handling & edge cases
14. ✅ Testing & bug fixes
15. ✅ Documentation & demo preparation

## Arcium-Specific Requirements

### MPC Functions Needed
1. **`mpc_shuffle_deck()`** - Secure deck shuffling
2. **`mpc_deal_card(player_pubkey)`** - Encrypted card dealing
3. **`mpc_reveal_card(card_index)`** - Decrypt specific card
4. **`mpc_generate_random()`** - Random number generation
5. **`mpc_verify_hand(encrypted_hand)`** - Hand validation

### Encrypted State Storage
- **Game State Account**
  - Encrypted deck state
  - Player encrypted hands
  - Community card indices
  - Current game phase

- **Player State Account**
  - Encrypted hole cards
  - Chip balance
  - Current bet amount
  - Action history

## Success Criteria for Hackathon

✅ **Must Have:**
- Functional 4-player poker game
- Arcium MPC integration for card encryption
- Real USDC/SOL betting
- Complete hand from deal to showdown
- Clear demonstration of privacy (hidden cards)

🎯 **Nice to Have:**
- 6-player support
- Side pot calculations
- Tournament mode
- Slick UI matching PokerStars

🔥 **Winning Edge:**
- Live demo during judging
- Clear explanation of Arcium's role
- Smooth gameplay with no bugs
- Professional UI/UX
- Open-source code with good documentation
