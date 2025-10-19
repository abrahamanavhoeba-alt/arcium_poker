use anchor_lang::prelude::*;
use crate::game::state::Game;
use crate::player::state::PlayerState;
use crate::shared::PokerError;

/// Tournament configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct TournamentConfig {
    /// Starting small blind
    pub starting_small_blind: u64,
    
    /// Starting big blind
    pub starting_big_blind: u64,
    
    /// Blind increase interval (in seconds)
    pub blind_increase_interval: i64,
    
    /// Blind increase multiplier (e.g., 2 = double blinds)
    pub blind_multiplier: u8,
    
    /// Total number of players
    pub total_players: u16,
    
    /// Players remaining
    pub players_remaining: u16,
    
    /// Final table size
    pub final_table_size: u8,
    
    /// Tournament started timestamp
    pub started_at: i64,
    
    /// Last blind increase timestamp
    pub last_blind_increase: i64,
    
    /// Current blind level
    pub blind_level: u8,
    
    /// Is tournament active
    pub is_active: bool,
}

impl Default for TournamentConfig {
    fn default() -> Self {
        Self {
            starting_small_blind: 100,
            starting_big_blind: 200,
            blind_increase_interval: 600, // 10 minutes
            blind_multiplier: 2,
            total_players: 0,
            players_remaining: 0,
            final_table_size: 9,
            started_at: 0,
            last_blind_increase: 0,
            blind_level: 1,
            is_active: false,
        }
    }
}

/// Tournament state
#[account]
pub struct TournamentState {
    /// Tournament ID
    pub tournament_id: u64,
    
    /// Configuration
    pub config: TournamentConfig,
    
    /// Prize pool
    pub prize_pool: u64,
    
    /// Eliminated players
    pub eliminated_players: Vec<Pubkey>,
    
    /// Placement tracking (player -> position)
    pub placements: Vec<(Pubkey, u16)>,
    
    /// Bump seed
    pub bump: u8,
}

impl TournamentState {
    pub const LEN: usize = 8 + // discriminator
        8 + // tournament_id
        std::mem::size_of::<TournamentConfig>() +
        8 + // prize_pool
        4 + (32 * 100) + // eliminated_players (max 100)
        4 + ((32 + 2) * 100) + // placements (max 100)
        1; // bump
}

/// Initialize tournament
pub fn initialize_tournament(
    tournament_state: &mut TournamentState,
    tournament_id: u64,
    config: TournamentConfig,
    bump: u8,
) -> Result<()> {
    tournament_state.tournament_id = tournament_id;
    tournament_state.config = config;
    tournament_state.prize_pool = 0;
    tournament_state.eliminated_players = Vec::new();
    tournament_state.placements = Vec::new();
    tournament_state.bump = bump;
    
    msg!(
        "[TOURNAMENT] Initialized tournament {} with {} players",
        tournament_id,
        config.total_players
    );
    
    Ok(())
}

/// Check if blinds should increase
pub fn should_increase_blinds(
    tournament_state: &TournamentState,
    current_time: i64,
) -> bool {
    if !tournament_state.config.is_active {
        return false;
    }
    
    let time_since_increase = current_time - tournament_state.config.last_blind_increase;
    time_since_increase >= tournament_state.config.blind_increase_interval
}

/// Increase blinds
pub fn increase_blinds(
    tournament_state: &mut TournamentState,
    game: &mut Game,
    current_time: i64,
) -> Result<()> {
    require!(
        tournament_state.config.is_active,
        PokerError::InvalidGameStage
    );
    
    // Calculate new blinds
    let multiplier = tournament_state.config.blind_multiplier as u64;
    game.small_blind = game.small_blind * multiplier;
    game.big_blind = game.big_blind * multiplier;
    
    // Update tournament state
    tournament_state.config.last_blind_increase = current_time;
    tournament_state.config.blind_level += 1;
    
    msg!(
        "[TOURNAMENT] Blinds increased to {}/{}. Level: {}",
        game.small_blind,
        game.big_blind,
        tournament_state.config.blind_level
    );
    
    Ok(())
}

/// Eliminate player from tournament
pub fn eliminate_player(
    tournament_state: &mut TournamentState,
    player: Pubkey,
) -> Result<()> {
    // Add to eliminated list
    tournament_state.eliminated_players.push(player);
    
    // Record placement
    let placement = tournament_state.config.players_remaining;
    tournament_state.placements.push((player, placement));
    
    // Decrease remaining count
    tournament_state.config.players_remaining -= 1;
    
    msg!(
        "[TOURNAMENT] Player {} eliminated. Placement: {}. Remaining: {}",
        player,
        placement,
        tournament_state.config.players_remaining
    );
    
    Ok(())
}

/// Check if final table should be consolidated
pub fn should_consolidate_final_table(tournament_state: &TournamentState) -> bool {
    tournament_state.config.players_remaining <= tournament_state.config.final_table_size as u16
}

/// Consolidate to final table
pub fn consolidate_final_table(
    tournament_state: &mut TournamentState,
) -> Result<()> {
    require!(
        should_consolidate_final_table(tournament_state),
        PokerError::InvalidGameStage
    );
    
    msg!(
        "[TOURNAMENT] Final table! {} players remaining",
        tournament_state.config.players_remaining
    );
    
    Ok(())
}

/// Calculate tournament payout
pub fn calculate_tournament_payout(
    tournament_state: &TournamentState,
    placement: u16,
) -> u64 {
    let prize_pool = tournament_state.prize_pool;
    
    // Standard tournament payout structure
    // 1st: 50%, 2nd: 30%, 3rd: 20%
    match placement {
        1 => prize_pool * 50 / 100,
        2 => prize_pool * 30 / 100,
        3 => prize_pool * 20 / 100,
        _ => 0,
    }
}

/// Get blind schedule
pub fn get_blind_schedule(level: u8, starting_blind: u64, multiplier: u8) -> (u64, u64) {
    let factor = (multiplier as u64).pow((level - 1) as u32);
    let small_blind = starting_blind * factor;
    let big_blind = small_blind * 2;
    
    (small_blind, big_blind)
}

/// Check if tournament is complete
pub fn is_tournament_complete(tournament_state: &TournamentState) -> bool {
    tournament_state.config.players_remaining <= 1
}
