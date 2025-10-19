use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::player::state::PlayerState;
use crate::game::state::Game;
use crate::shared::PokerError;
use super::conversion::{ConversionRate, chips_to_tokens};
use super::escrow::release_tokens_on_leave;

/// Withdraw chips to tokens
pub fn withdraw_chips_to_tokens<'info>(
    player_state: &mut PlayerState,
    game_key: Pubkey,
    escrow_token_account: &Account<'info, TokenAccount>,
    player_token_account: &Account<'info, TokenAccount>,
    escrow_authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    chip_amount: u64,
    conversion_rate: &ConversionRate,
    escrow_bump: u8,
    rake_percentage: u8,
) -> Result<()> {
    // Validate player has sufficient chips
    require!(
        player_state.chip_stack >= chip_amount,
        PokerError::InsufficientChips
    );
    
    // Calculate token amount
    let token_amount = chips_to_tokens(chip_amount, conversion_rate);
    
    // Calculate and deduct fee if applicable
    let (net_amount, fee) = calculate_withdrawal_fee(token_amount, rake_percentage);
    
    // Deduct chips from player
    player_state.chip_stack -= chip_amount;
    
    // Transfer tokens to player
    release_tokens_on_leave(
        escrow_token_account,
        player_token_account,
        escrow_authority,
        token_program,
        net_amount,
        escrow_bump,
        game_key,
    )?;
    
    msg!(
        "[TOKEN] Player {} withdrew {} chips ({} tokens, {} fee)",
        player_state.player,
        chip_amount,
        net_amount,
        fee
    );
    
    Ok(())
}

/// Calculate withdrawal fee (rake)
pub fn calculate_withdrawal_fee(amount: u64, rake_percentage: u8) -> (u64, u64) {
    if rake_percentage == 0 {
        return (amount, 0);
    }
    
    let fee = (amount * rake_percentage as u64) / 100;
    let net_amount = amount.saturating_sub(fee);
    
    (net_amount, fee)
}

/// Instant settlement after hand completion
pub fn settle_hand_winnings<'info>(
    player_states: &mut [PlayerState],
    game_key: Pubkey,
    escrow_token_account: &Account<'info, TokenAccount>,
    player_token_accounts: &[Account<'info, TokenAccount>],
    escrow_authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    conversion_rate: &ConversionRate,
    escrow_bump: u8,
) -> Result<()> {
    for (i, player_state) in player_states.iter_mut().enumerate() {
        if player_state.chip_stack == 0 {
            continue;
        }
        
        // Auto-cashout players who are leaving
        if player_state.status == crate::types::PlayerStatus::Left {
            let token_amount = chips_to_tokens(player_state.chip_stack, conversion_rate);
            
            release_tokens_on_leave(
                escrow_token_account,
                &player_token_accounts[i],
                escrow_authority,
                token_program,
                token_amount,
                escrow_bump,
                game_key,
            )?;
            
            player_state.chip_stack = 0;
            
            msg!(
                "[TOKEN] Auto-settled {} tokens for player {}",
                token_amount,
                player_state.player
            );
        }
    }
    
    Ok(())
}

/// Validate withdrawal request
pub fn validate_withdrawal(
    player_state: &PlayerState,
    game: &Game,
    chip_amount: u64,
) -> Result<()> {
    // Cannot withdraw during active hand
    require!(
        game.stage == crate::types::GameStage::Waiting || 
        game.stage == crate::types::GameStage::Finished,
        PokerError::InvalidGameStage
    );
    
    // Must have sufficient chips
    require!(
        player_state.chip_stack >= chip_amount,
        PokerError::InsufficientChips
    );
    
    // Cannot withdraw if in active hand
    require!(
        !player_state.has_cards || player_state.has_folded,
        PokerError::InvalidAction
    );
    
    Ok(())
}
