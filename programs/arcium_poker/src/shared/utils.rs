use anchor_lang::prelude::*;
use super::constants::*;
use super::errors::PokerError;

/// Validate buy-in amount
pub fn validate_buy_in(amount: u64, min_buy_in: u64, max_buy_in: u64) -> Result<()> {
    require!(amount >= min_buy_in, PokerError::BuyInTooLow);
    require!(amount <= max_buy_in, PokerError::BuyInTooHigh);
    Ok(())
}

/// Find next active player index
pub fn find_next_active_player(
    current_index: usize,
    active_players: &[bool],
    player_count: usize,
) -> Option<usize> {
    for i in 1..=player_count {
        let next_index = (current_index + i) % player_count;
        if active_players[next_index] {
            return Some(next_index);
        }
    }
    None
}

/// Calculate pot total from all contributions
pub fn calculate_pot_total(contributions: &[u64]) -> u64 {
    contributions.iter().sum()
}