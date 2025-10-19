// Pot distribution and payout logic
// To be implemented in Module 4

use anchor_lang::prelude::*;
use crate::game::state::Game;
use crate::player::state::PlayerState;
use crate::shared::PokerError;

/// Distribute winnings to winners
pub fn distribute_winnings(
    game: &mut Game,
    player_states: &mut [PlayerState],
    winners: &[(u8, u64)], // (seat_index, amount)
) -> Result<()> {
    let mut total_distributed = 0u64;
    
    for (seat_index, amount) in winners {
        let player_state = &mut player_states[*seat_index as usize];
        
        // Add winnings to player's chip stack
        player_state.add_winnings(*amount);
        total_distributed += amount;
        
        msg!(
            "[PAYOUT] Seat {} received {} chips. New stack: {}",
            seat_index,
            amount,
            player_state.chip_stack
        );
    }
    
    // Verify total distributed matches pot
    require!(
        total_distributed <= game.pot,
        PokerError::InvalidGameConfig
    );
    
    // Reset pot
    game.pot = 0;
    
    msg!("[PAYOUT] Total distributed: {}", total_distributed);
    
    Ok(())
}

/// Transfer winnings from game PDA to player accounts (for SOL/tokens)
pub fn transfer_winnings_to_accounts(
    game_account: &AccountInfo,
    player_accounts: &[AccountInfo],
    winners: &[(u8, u64)],
) -> Result<()> {
    for (seat_index, amount) in winners {
        let player_account = &player_accounts[*seat_index as usize];
        
        // Transfer lamports from game PDA to player
        **game_account.try_borrow_mut_lamports()? -= amount;
        **player_account.try_borrow_mut_lamports()? += amount;
        
        msg!(
            "[PAYOUT] Transferred {} lamports to seat {}",
            amount,
            seat_index
        );
    }
    
    Ok(())
}

/// Handle rake (house fee) - optional
pub fn calculate_rake(pot_amount: u64, rake_percentage: u8) -> u64 {
    // Rake is typically 2.5-5% of pot, capped at a maximum
    let rake = (pot_amount * rake_percentage as u64) / 100;
    let max_rake = 3_000_000; // 0.003 SOL max rake
    rake.min(max_rake)
}

/// Distribute pot with rake
pub fn distribute_with_rake(
    game: &mut Game,
    player_states: &mut [PlayerState],
    winners: &[(u8, u64)],
    rake_percentage: u8,
    house_account: &AccountInfo,
    game_account: &AccountInfo,
) -> Result<()> {
    // Calculate and deduct rake
    let rake = calculate_rake(game.pot, rake_percentage);
    
    if rake > 0 {
        // Transfer rake to house
        **game_account.try_borrow_mut_lamports()? -= rake;
        **house_account.try_borrow_mut_lamports()? += rake;
        
        msg!("[PAYOUT] Rake collected: {}", rake);
    }
    
    // Distribute remaining pot
    distribute_winnings(game, player_states, winners)?;
    
    Ok(())
}