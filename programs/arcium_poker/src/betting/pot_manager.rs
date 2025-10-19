// Pot calculation and side pot management
// To be implemented in Module 3

use anchor_lang::prelude::*;
use super::state::SidePot;
use crate::shared::constants::MAX_PLAYERS;
use crate::shared::PokerError;

const MAX_SIDE_POTS: usize = 6; // Max side pots = max players

/// Pot manager for handling main pot and side pots
pub struct PotManager {
    /// Main pot amount
    pub main_pot: u64,
    
    /// Side pots for all-in scenarios
    pub side_pots: [SidePot; MAX_SIDE_POTS],
    
    /// Number of active side pots
    pub side_pot_count: u8,
    
    /// Player contributions in current round
    pub player_contributions: [u64; MAX_PLAYERS],
}

impl PotManager {
    pub fn new() -> Self {
        Self {
            main_pot: 0,
            side_pots: [SidePot::default(); MAX_SIDE_POTS],
            side_pot_count: 0,
            player_contributions: [0; MAX_PLAYERS],
        }
    }
    
    /// Add bet to pot
    pub fn add_bet(&mut self, seat_index: usize, amount: u64) {
        self.player_contributions[seat_index] += amount;
        self.main_pot += amount;
    }
    
    /// Calculate and create side pots for all-in scenarios
    /// This should be called at the end of each betting round
    pub fn calculate_side_pots(
        &mut self,
        player_count: usize,
        all_in_players: &[bool; MAX_PLAYERS],
        active_players: &[bool; MAX_PLAYERS],
    ) -> Result<()> {
        // Reset side pots
        self.side_pots = [SidePot::default(); MAX_SIDE_POTS];
        self.side_pot_count = 0;
        
        // If no all-ins, everything goes to main pot
        if !all_in_players.iter().any(|&x| x) {
            return Ok(());
        }
        
        // Collect all-in amounts and sort them
        let mut all_in_amounts: Vec<(usize, u64)> = Vec::new();
        for i in 0..player_count {
            if all_in_players[i] {
                all_in_amounts.push((i, self.player_contributions[i]));
            }
        }
        all_in_amounts.sort_by_key(|&(_, amount)| amount);
        
        let mut previous_level = 0u64;
        
        // Create side pots for each all-in level
        for (all_in_seat, all_in_amount) in all_in_amounts.iter() {
            if *all_in_amount <= previous_level {
                continue;
            }
            
            let level_contribution = all_in_amount - previous_level;
            let mut pot_amount = 0u64;
            let mut side_pot = SidePot::new(0);
            
            // Calculate pot amount and eligible players
            for i in 0..player_count {
                if active_players[i] && self.player_contributions[i] >= *all_in_amount {
                    pot_amount += level_contribution;
                    side_pot.add_eligible_player(i);
                }
            }
            
            if pot_amount > 0 {
                side_pot.amount = pot_amount;
                require!(
                    (self.side_pot_count as usize) < MAX_SIDE_POTS,
                    PokerError::InvalidGameConfig
                );
                self.side_pots[self.side_pot_count as usize] = side_pot;
                self.side_pot_count += 1;
            }
            
            previous_level = *all_in_amount;
        }
        
        // Remaining goes to main pot (for players not all-in)
        let mut main_pot_amount = 0u64;
        for i in 0..player_count {
            if active_players[i] && self.player_contributions[i] > previous_level {
                main_pot_amount += self.player_contributions[i] - previous_level;
            }
        }
        
        // Adjust main pot
        self.main_pot = main_pot_amount;
        
        Ok(())
    }
    
    /// Get total pot (main + all side pots)
    pub fn get_total_pot(&self) -> u64 {
        let mut total = self.main_pot;
        for i in 0..self.side_pot_count as usize {
            total += self.side_pots[i].amount;
        }
        total
    }
    
    /// Reset for new betting round
    pub fn reset_for_new_round(&mut self) {
        self.player_contributions = [0; MAX_PLAYERS];
    }
    
    /// Reset for new hand
    pub fn reset_for_new_hand(&mut self) {
        self.main_pot = 0;
        self.side_pots = [SidePot::default(); MAX_SIDE_POTS];
        self.side_pot_count = 0;
        self.player_contributions = [0; MAX_PLAYERS];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_pot() {
        let mut pot_manager = PotManager::new();
        pot_manager.add_bet(0, 100);
        pot_manager.add_bet(1, 100);
        assert_eq!(pot_manager.main_pot, 200);
    }
    
    #[test]
    fn test_side_pot_creation() {
        let mut pot_manager = PotManager::new();
        pot_manager.add_bet(0, 50);  // All-in
        pot_manager.add_bet(1, 100);
        pot_manager.add_bet(2, 100);
        
        let mut all_in = [false; MAX_PLAYERS];
        all_in[0] = true;
        let mut active = [false; MAX_PLAYERS];
        active[0] = true;
        active[1] = true;
        active[2] = true;
        
        pot_manager.calculate_side_pots(3, &all_in, &active).unwrap();
        
        // Side pot 0: 50 * 3 = 150 (all players eligible)
        assert_eq!(pot_manager.side_pots[0].amount, 150);
        assert_eq!(pot_manager.side_pots[0].player_count, 3);
        
        // Main pot: (100-50) * 2 = 100 (only players 1 and 2 eligible)
        assert_eq!(pot_manager.main_pot, 100);
    }
}