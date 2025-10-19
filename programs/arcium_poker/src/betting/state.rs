use anchor_lang::prelude::*;
use crate::shared::constants::MAX_PLAYERS;

/// Side pot structure for all-in scenarios
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
pub struct SidePot {
    /// Total amount in this side pot
    pub amount: u64,
    
    /// Players eligible to win this pot (bitmap)
    pub eligible_players: [bool; MAX_PLAYERS],
    
    /// Number of eligible players
    pub player_count: u8,
}

impl SidePot {
    pub fn new(amount: u64) -> Self {
        Self {
            amount,
            eligible_players: [false; MAX_PLAYERS],
            player_count: 0,
        }
    }
    
    pub fn add_eligible_player(&mut self, seat_index: usize) {
        if !self.eligible_players[seat_index] {
            self.eligible_players[seat_index] = true;
            self.player_count += 1;
        }
    }
    
    pub fn is_eligible(&self, seat_index: usize) -> bool {
        self.eligible_players[seat_index]
    }
}

/// Betting round state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BettingRound {
    PreFlop,
    Flop,
    Turn,
    River,
}

/// Player action in current betting round
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerBetAction {
    /// Amount bet in current round
    pub current_round_bet: u64,
    
    /// Total bet in current hand
    pub total_hand_bet: u64,
    
    /// Has acted in current round
    pub has_acted: bool,
    
    /// Has folded
    pub has_folded: bool,
    
    /// Is all-in
    pub is_all_in: bool,
}

impl Default for PlayerBetAction {
    fn default() -> Self {
        Self {
            current_round_bet: 0,
            total_hand_bet: 0,
            has_acted: false,
            has_folded: false,
            is_all_in: false,
        }
    }
}

impl PlayerBetAction {
    pub fn reset_for_new_round(&mut self) {
        self.current_round_bet = 0;
        self.has_acted = false;
    }
    
    pub fn reset_for_new_hand(&mut self) {
        self.current_round_bet = 0;
        self.total_hand_bet = 0;
        self.has_acted = false;
        self.has_folded = false;
        self.is_all_in = false;
    }
}