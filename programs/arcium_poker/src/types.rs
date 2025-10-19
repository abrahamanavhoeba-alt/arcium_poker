use anchor_lang::prelude::*;

/// Game stage/phase
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameStage {
    Waiting,        // Waiting for players
    PreFlop,        // Hole cards dealt, betting round 1
    Flop,           // 3 community cards revealed
    Turn,           // 4th community card revealed
    River,          // 5th community card revealed
    Showdown,       // Reveal hands and determine winner
    Finished,       // Game completed
}

impl Default for GameStage {
    fn default() -> Self {
        GameStage::Waiting
    }
}

/// Player action types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerAction {
    Fold,
    Check,
    Call,
    Raise,
    AllIn,
}

/// Player action parameter for unified action handler
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum PlayerActionParam {
    Fold,
    Check,
    Call,
    Bet { amount: u64 },
    Raise { amount: u64 },
    AllIn,
}

/// Player status in current hand
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerStatus {
    Waiting,        // Joined but game not started
    Active,         // In the hand
    Folded,         // Folded this hand
    AllIn,          // All-in
    Left,           // Left the game
}

impl Default for PlayerStatus {
    fn default() -> Self {
        PlayerStatus::Waiting
    }
}

/// Card suit
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

/// Card rank
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

/// Poker hand rankings
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum HandRank {
    HighCard = 0,
    OnePair = 1,
    TwoPair = 2,
    ThreeOfAKind = 3,
    Straight = 4,
    Flush = 5,
    FullHouse = 6,
    FourOfAKind = 7,
    StraightFlush = 8,
    RoyalFlush = 9,
}