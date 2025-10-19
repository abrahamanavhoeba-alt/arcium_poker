use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use super::state::PlayerState;
use crate::game::state::Game;
use crate::types::GameStage;
use crate::shared::{PokerError, validate_buy_in};

/// Player joins a poker game
pub fn handler(ctx: Context<crate::JoinGame>, buy_in: u64) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let player_state = &mut ctx.accounts.player_state;
    
    // Validate game state
    require!(
        game.stage == GameStage::Waiting,
        PokerError::GameAlreadyStarted
    );
    
    // Validate buy-in amount
    validate_buy_in(buy_in, game.min_buy_in, game.max_buy_in)?;
    
    // Add player to game and get seat index
    let seat_index = game.add_player(ctx.accounts.player.key())?;
    
    // Store values we need for later (before transfers)
    let game_key = game.key();
    let game_id = game.game_id;
    let player_count = game.player_count;
    let max_players = game.max_players;
    let player_key = ctx.accounts.player.key();
    
    // Transfer buy-in to game escrow (game PDA)
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.player.to_account_info(),
                to: game.to_account_info(),
            },
        ),
        buy_in,
    )?;
    
    // Initialize player state
    player_state.initialize(
        player_key,
        game_key,
        seat_index,
        buy_in,
        ctx.bumps.player_state,
    );
    
    msg!(
        "Player {} joined game {} at seat {} with {} chips",
        player_key,
        game_id,
        seat_index,
        buy_in
    );
    msg!("Players in game: {}/{}", player_count, max_players);
    
    Ok(())
}

// JoinGame struct moved to lib.rs at crate root (required by Anchor)