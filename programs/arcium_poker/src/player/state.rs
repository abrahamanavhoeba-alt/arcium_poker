use anchor_lang::prelude::*;
use crate::types::PlayerStatus;
use crate::shared::constants::HOLE_CARDS;

/// Player state account (PDA per player per game)
#[account]
pub struct PlayerState {
    /// Player's public key
    pub player: Pubkey,
    
    /// Game this player belongs to
    pub game: Pubkey,
    
    /// Player's seat index in the game
    pub seat_index: u8,
    
    /// Player status
    pub status: PlayerStatus,
    
    /// Player's chip stack
    pub chip_stack: u64,
    
    /// Current bet in this round
    pub current_bet: u64,
    
    /// Total contribution to pot this hand
    pub total_bet_this_hand: u64,
    
    /// Encrypted hole cards (indices in deck)
    pub encrypted_hole_cards: [u8; HOLE_CARDS],
    
    /// Has cards been dealt to this player
    pub has_cards: bool,
    
    /// Player folded in current hand
    pub has_folded: bool,
    
    /// Player is all-in
    pub is_all_in: bool,
    
    /// Timestamp when player joined
    pub joined_at: i64,
    
    /// Last action timestamp
    pub last_action_at: i64,
    
    /// Bump seed for PDA
    pub bump: u8,
}

impl PlayerState {
    /// Calculate space needed for PlayerState account
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        32 + // game
        1 + // seat_index
        1 + // status
        8 + // chip_stack
        8 + // current_bet
        8 + // total_bet_this_hand
        (1 * HOLE_CARDS) + // encrypted_hole_cards
        1 + // has_cards
        1 + // has_folded
        1 + // is_all_in
        8 + // joined_at
        8 + // last_action_at
        1; // bump
    
    /// Initialize player state
    pub fn initialize(
        &mut self,
        player: Pubkey,
        game: Pubkey,
        seat_index: u8,
        buy_in: u64,
        bump: u8,
    ) {
        self.player = player;
        self.game = game;
        self.seat_index = seat_index;
        self.status = PlayerStatus::Waiting;
        self.chip_stack = buy_in;
        self.current_bet = 0;
        self.total_bet_this_hand = 0;
        self.encrypted_hole_cards = [0; HOLE_CARDS];
        self.has_cards = false;
        self.has_folded = false;
        self.is_all_in = false;
        self.joined_at = Clock::get().unwrap().unix_timestamp;
        self.last_action_at = Clock::get().unwrap().unix_timestamp;
        self.bump = bump;
    }
    
    /// Place a bet
    pub fn place_bet(&mut self, amount: u64) -> Result<()> {
        require!(
            self.chip_stack >= amount,
            crate::shared::PokerError::InsufficientChips
        );
        
        self.chip_stack -= amount;
        self.current_bet += amount;
        self.total_bet_this_hand += amount;
        
        // Check if all-in
        if self.chip_stack == 0 {
            self.is_all_in = true;
        }
        
        self.last_action_at = Clock::get().unwrap().unix_timestamp;
        
        Ok(())
    }
    
    /// Fold hand
    pub fn fold(&mut self) {
        self.has_folded = true;
        self.status = PlayerStatus::Folded;
        self.last_action_at = Clock::get().unwrap().unix_timestamp;
    }
    
    /// Reset for new round
    pub fn reset_for_new_round(&mut self) {
        self.current_bet = 0;
    }
    
    /// Reset for new hand
    pub fn reset_for_new_hand(&mut self) {
        self.current_bet = 0;
        self.total_bet_this_hand = 0;
        self.encrypted_hole_cards = [0; HOLE_CARDS];
        self.has_cards = false;
        self.has_folded = false;
        self.is_all_in = false;
        
        if self.chip_stack > 0 {
            self.status = PlayerStatus::Active;
        }
    }
    
    /// Add winnings
    pub fn add_winnings(&mut self, amount: u64) {
        self.chip_stack += amount;
    }
}