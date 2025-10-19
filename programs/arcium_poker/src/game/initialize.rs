use anchor_lang::prelude::*;
use super::state::Game;
use crate::shared::{constants::*, PokerError};

/// Initialize a new poker game
pub fn handler(
    ctx: Context<crate::InitializeGame>,
    game_id: u64,
    small_blind: Option<u64>,
    big_blind: Option<u64>,
    min_buy_in: Option<u64>,
    max_buy_in: Option<u64>,
    max_players: Option<u8>,
) -> Result<()> {
    let game = &mut ctx.accounts.game;
    
    // Use defaults if not provided
    let small_blind = small_blind.unwrap_or(DEFAULT_SMALL_BLIND);
    let big_blind = big_blind.unwrap_or(DEFAULT_BIG_BLIND);
    let min_buy_in = min_buy_in.unwrap_or(MIN_BUY_IN);
    let max_buy_in = max_buy_in.unwrap_or(MAX_BUY_IN);
    let max_players_val = max_players.unwrap_or(MAX_PLAYERS as u8);
    
    // Validate configuration
    require!(
        max_players_val >= MIN_PLAYERS as u8 && max_players_val <= MAX_PLAYERS as u8,
        PokerError::InvalidGameConfig
    );
    require!(big_blind > small_blind, PokerError::InvalidGameConfig);
    require!(min_buy_in >= big_blind * 50, PokerError::InvalidGameConfig); // At least 50 BBs
    require!(max_buy_in >= min_buy_in, PokerError::InvalidGameConfig);
    
    // Initialize game
    let initialized_game = Game::new(
        game_id,
        ctx.accounts.authority.key(),
        small_blind,
        big_blind,
        min_buy_in,
        max_buy_in,
        max_players_val,
        ctx.bumps.game,
    )?;
    
    **game = initialized_game;
    
    msg!("Game {} initialized by {}", game_id, ctx.accounts.authority.key());
    msg!("Blinds: {}/{}, Buy-in: {}-{}", small_blind, big_blind, min_buy_in, max_buy_in);
    
    Ok(())
}

// InitializeGame struct moved to lib.rs at crate root (required by Anchor)