use anchor_lang::prelude::*;
use super::state::PlayerState;
use crate::game::state::Game;
use crate::types::GameStage;
use crate::shared::PokerError;

/// Player leaves a poker game
pub fn handler(ctx: Context<crate::LeaveGame>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let player_state = &ctx.accounts.player_state;
    
    // Cannot leave during active hand (after PreFlop starts)
    require!(
        game.stage == GameStage::Waiting || game.stage == GameStage::Finished,
        PokerError::CannotLeaveDuringHand
    );
    
    // Verify player is in this game
    require!(
        player_state.game == game.key(),
        PokerError::PlayerNotInGame
    );
    
    // Remove player from game
    game.remove_player(&ctx.accounts.player.key())?;
    
    // Return remaining chips to player
    let remaining_chips = player_state.chip_stack;
    if remaining_chips > 0 {
        // Transfer lamports from game PDA to player
        **game.to_account_info().try_borrow_mut_lamports()? -= remaining_chips;
        **ctx.accounts.player.to_account_info().try_borrow_mut_lamports()? += remaining_chips;
        
        msg!("Returned {} chips to player", remaining_chips);
    }
    
    msg!(
        "Player {} left game {}",
        ctx.accounts.player.key(),
        game.game_id
    );
    msg!("Players remaining: {}", game.player_count);
    
    Ok(())
}

// LeaveGame struct moved to lib.rs at crate root (required by Anchor)