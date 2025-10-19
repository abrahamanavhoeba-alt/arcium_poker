use anchor_lang::prelude::*;
use crate::types::GameStage;
use crate::shared::constants::*;

/// Main game account
#[account]
pub struct Game {
    /// Game authority (creator)
    pub authority: Pubkey,
    
    /// Unique game ID
    pub game_id: u64,
    
    /// Current game stage
    pub stage: GameStage,
    
    /// Small blind amount
    pub small_blind: u64,
    
    /// Big blind amount
    pub big_blind: u64,
    
    /// Minimum buy-in
    pub min_buy_in: u64,
    
    /// Maximum buy-in
    pub max_buy_in: u64,
    
    /// Maximum number of players (4-6)
    pub max_players: u8,
    
    /// Current number of players
    pub player_count: u8,
    
    /// Player public keys (seats)
    pub players: [Pubkey; MAX_PLAYERS],
    
    /// Active player flags
    pub active_players: [bool; MAX_PLAYERS],
    
    /// Current dealer button position
    pub dealer_position: u8,
    
    /// Current active player (whose turn it is)
    pub current_player_index: u8,
    
    /// Total pot amount
    pub pot: u64,
    
    /// Current bet amount in this round
    pub current_bet: u64,
    
    /// Players who have acted in current betting round
    pub players_acted: [bool; MAX_PLAYERS],
    
    /// Community cards (encrypted indices)
    pub community_cards: [u8; COMMUNITY_CARDS],
    
    /// Number of community cards revealed
    pub community_cards_revealed: u8,
    
    /// Encrypted deck state (managed by Arcium MPC)
    pub encrypted_deck: [u8; 32], // Hash or reference to encrypted deck
    
    /// Deck initialized flag
    pub deck_initialized: bool,
    
    /// Game started timestamp
    pub started_at: i64,
    
    /// Last action timestamp
    pub last_action_at: i64,
    
    /// Shuffle session ID from Arcium MPC
    pub shuffle_session_id: [u8; 32],
    
    /// Game bump seed
    pub bump: u8,
}

impl Game {
    /// Calculate space needed for Game account
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 + // game_id
        1 + // stage
        8 + // small_blind
        8 + // big_blind
        8 + // min_buy_in
        8 + // max_buy_in
        1 + // max_players
        1 + // player_count
        (32 * MAX_PLAYERS) + // players
        (1 * MAX_PLAYERS) + // active_players
        1 + // dealer_position
        1 + // current_player_index
        8 + // pot
        8 + // current_bet
        (1 * MAX_PLAYERS) + // players_acted
        (1 * COMMUNITY_CARDS) + // community_cards
        1 + // community_cards_revealed
        32 + // encrypted_deck
        1 + // deck_initialized
        8 + // started_at
        8 + // last_action_at
        32 + // shuffle_session_id
        1; // bump
    
    /// Initialize game with default values
    pub fn new(
        game_id: u64,
        authority: Pubkey,
        small_blind: u64,
        big_blind: u64,
        min_buyin: u64,
        max_buyin: u64,
        max_players: u8,
        bump: u8,
    ) -> Result<Self> {
        let game = Self {
            authority,
            game_id,
            stage: GameStage::Waiting,
            small_blind,
            big_blind,
            min_buy_in: min_buyin,
            max_buy_in: max_buyin,
            max_players,
            player_count: 0,
            players: [Pubkey::default(); MAX_PLAYERS],
            active_players: [false; MAX_PLAYERS],
            dealer_position: 0,
            current_player_index: 0,
            pot: 0,
            current_bet: 0,
            players_acted: [false; MAX_PLAYERS],
            community_cards: [0; COMMUNITY_CARDS],
            community_cards_revealed: 0,
            encrypted_deck: [0; 32],
            deck_initialized: false,
            started_at: 0,
            last_action_at: Clock::get()?.unix_timestamp,
            shuffle_session_id: [0; 32],
            bump,
        };
        Ok(game)
    }
    
    /// Check if game is full
    pub fn is_full(&self) -> bool {
        self.player_count >= self.max_players
    }
    
    /// Check if player is in game
    pub fn has_player(&self, player: &Pubkey) -> bool {
        self.players[..self.player_count as usize]
            .iter()
            .any(|p| p == player)
    }
    
    /// Add player to game
    pub fn add_player(&mut self, player: Pubkey) -> Result<u8> {
        require!(!self.is_full(), crate::shared::PokerError::GameFull);
        require!(!self.has_player(&player), crate::shared::PokerError::PlayerAlreadyInGame);
        
        let seat_index = self.player_count;
        self.players[seat_index as usize] = player;
        self.active_players[seat_index as usize] = true;
        self.player_count += 1;
        
        Ok(seat_index)
    }
    
    /// Remove player from game
    pub fn remove_player(&mut self, player: &Pubkey) -> Result<()> {
        let player_index = self.players[..self.player_count as usize]
            .iter()
            .position(|p| p == player)
            .ok_or(crate::shared::PokerError::PlayerNotInGame)?;
        
        self.active_players[player_index] = false;
        
        // If game hasn't started, we can actually remove them
        if self.stage == GameStage::Waiting {
            // Shift players down
            for i in player_index..(self.player_count as usize - 1) {
                self.players[i] = self.players[i + 1];
                self.active_players[i] = self.active_players[i + 1];
            }
            self.players[self.player_count as usize - 1] = Pubkey::default();
            self.active_players[self.player_count as usize - 1] = false;
            self.player_count -= 1;
        }
        
        Ok(())
    }
    
    /// Get encrypted deck (for dealing cards)
    /// Note: This is a simplified accessor. In production, the encrypted deck
    /// would be stored in a separate account to handle larger data structures
    pub fn get_encrypted_deck(&self) -> Result<crate::cards::deck::EncryptedDeck> {
        // For MVP, we store deck reference in encrypted_deck field
        // In production, this would load from a separate PDA account
        require!(self.deck_initialized, crate::shared::PokerError::DeckNotInitialized);
        
        // TODO: Load actual encrypted deck from separate account
        // For now, create mock deck structure
        Ok(crate::cards::deck::EncryptedDeck::default())
    }
}