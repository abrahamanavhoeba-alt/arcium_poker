/// Maximum number of players per game
pub const MAX_PLAYERS: usize = 6;

/// Minimum number of players to start
pub const MIN_PLAYERS: usize = 2;

/// Number of hole cards per player
pub const HOLE_CARDS: usize = 2;

/// Number of community cards
pub const COMMUNITY_CARDS: usize = 5;

/// Total cards in deck
pub const DECK_SIZE: usize = 52;

/// Turn timeout in seconds
pub const TURN_TIMEOUT: i64 = 60;

/// Minimum raise multiplier
pub const MIN_RAISE_MULTIPLIER: u64 = 2;

/// Default small blind amount (in lamports/smallest unit)
pub const DEFAULT_SMALL_BLIND: u64 = 1_000_000; // 0.001 SOL or equivalent

/// Default big blind amount
pub const DEFAULT_BIG_BLIND: u64 = 2_000_000; // 0.002 SOL or equivalent

/// Minimum buy-in (100 big blinds)
pub const MIN_BUY_IN: u64 = 200_000_000; // 0.2 SOL or equivalent

/// Maximum buy-in (1000 big blinds)
pub const MAX_BUY_IN: u64 = 2_000_000_000; // 2 SOL or equivalent