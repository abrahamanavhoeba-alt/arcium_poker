use anchor_lang::prelude::*;
use crate::shared::PokerError;

/// Rake configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct RakeConfig {
    /// Rake percentage (in basis points, e.g., 250 = 2.5%)
    pub rake_percentage: u16,
    
    /// Maximum rake per hand (in lamports/tokens)
    pub rake_cap: u64,
    
    /// Minimum pot size to collect rake
    pub min_pot_for_rake: u64,
    
    /// House wallet for rake collection
    pub house_wallet: Pubkey,
    
    /// Total rake collected
    pub total_rake_collected: u64,
    
    /// Number of hands raked
    pub hands_raked: u64,
}

impl Default for RakeConfig {
    fn default() -> Self {
        Self {
            rake_percentage: 250, // 2.5%
            rake_cap: 3_000_000, // 0.003 SOL or 3 USDC
            min_pot_for_rake: 1_000_000, // 0.001 SOL or 1 USDC
            house_wallet: Pubkey::default(),
            total_rake_collected: 0,
            hands_raked: 0,
        }
    }
}

/// Calculate rake for a pot
pub fn calculate_rake(pot_amount: u64, config: &RakeConfig) -> u64 {
    // No rake if pot is too small
    if pot_amount < config.min_pot_for_rake {
        return 0;
    }
    
    // Calculate rake as percentage
    let rake = (pot_amount * config.rake_percentage as u64) / 10_000;
    
    // Apply cap
    rake.min(config.rake_cap)
}

/// Get rake for pot (returns net pot and rake amount)
pub fn get_rake_for_pot(pot_amount: u64, config: &RakeConfig) -> (u64, u64) {
    let rake = calculate_rake(pot_amount, config);
    let net_pot = pot_amount.saturating_sub(rake);
    
    (net_pot, rake)
}

/// Collect rake from pot
pub fn collect_rake(
    pot_amount: u64,
    config: &mut RakeConfig,
) -> Result<(u64, u64)> {
    let (net_pot, rake) = get_rake_for_pot(pot_amount, config);
    
    if rake > 0 {
        config.total_rake_collected += rake;
        config.hands_raked += 1;
        
        msg!(
            "[RAKE] Collected {} rake from {} pot. Total rake: {}",
            rake,
            pot_amount,
            config.total_rake_collected
        );
    }
    
    Ok((net_pot, rake))
}

/// Transfer rake to house wallet
pub fn transfer_rake_to_house(
    game_account: &AccountInfo,
    house_account: &AccountInfo,
    rake_amount: u64,
) -> Result<()> {
    require!(
        rake_amount > 0,
        PokerError::InvalidBetAmount
    );
    
    // Transfer lamports from game to house
    **game_account.try_borrow_mut_lamports()? -= rake_amount;
    **house_account.try_borrow_mut_lamports()? += rake_amount;
    
    msg!(
        "[RAKE] Transferred {} to house wallet",
        rake_amount
    );
    
    Ok(())
}

/// Calculate rake statistics
pub fn calculate_rake_stats(config: &RakeConfig) -> (u64, u64) {
    let average_rake = if config.hands_raked > 0 {
        config.total_rake_collected / config.hands_raked
    } else {
        0
    };
    
    (config.total_rake_collected, average_rake)
}

/// Validate rake configuration
pub fn validate_rake_config(config: &RakeConfig) -> Result<()> {
    // Rake percentage should be reasonable (max 10%)
    require!(
        config.rake_percentage <= 1000,
        PokerError::InvalidGameConfig
    );
    
    // Rake cap should be set
    require!(
        config.rake_cap > 0,
        PokerError::InvalidGameConfig
    );
    
    // House wallet should be valid
    require!(
        config.house_wallet != Pubkey::default(),
        PokerError::InvalidGameConfig
    );
    
    Ok(())
}

/// Calculate rake for different game types
pub fn calculate_rake_by_game_type(
    pot_amount: u64,
    game_type: GameType,
) -> u64 {
    let config = match game_type {
        GameType::CashGame => RakeConfig {
            rake_percentage: 250, // 2.5%
            rake_cap: 3_000_000,
            ..Default::default()
        },
        GameType::Tournament => RakeConfig {
            rake_percentage: 0, // No rake in tournaments (taken from buy-in)
            rake_cap: 0,
            ..Default::default()
        },
        GameType::SitAndGo => RakeConfig {
            rake_percentage: 500, // 5%
            rake_cap: 5_000_000,
            ..Default::default()
        },
    };
    
    calculate_rake(pot_amount, &config)
}

/// Game type enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameType {
    CashGame,
    Tournament,
    SitAndGo,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rake_calculation() {
        let config = RakeConfig {
            rake_percentage: 250, // 2.5%
            rake_cap: 3_000_000,
            min_pot_for_rake: 1_000_000,
            ..Default::default()
        };
        
        // Small pot - no rake
        assert_eq!(calculate_rake(500_000, &config), 0);
        
        // Normal pot - 2.5% rake
        assert_eq!(calculate_rake(100_000_000, &config), 2_500_000);
        
        // Large pot - capped at max
        assert_eq!(calculate_rake(200_000_000, &config), 3_000_000);
    }
    
    #[test]
    fn test_net_pot_calculation() {
        let config = RakeConfig {
            rake_percentage: 250,
            rake_cap: 3_000_000,
            min_pot_for_rake: 1_000_000,
            ..Default::default()
        };
        
        let (net_pot, rake) = get_rake_for_pot(100_000_000, &config);
        assert_eq!(net_pot, 97_500_000);
        assert_eq!(rake, 2_500_000);
    }
}
