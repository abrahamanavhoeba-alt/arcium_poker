use anchor_lang::prelude::*;

/// Conversion rate configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct ConversionRate {
    /// Tokens per chip (e.g., 1_000_000 = 1:1 for 6 decimal token)
    pub tokens_per_chip: u64,
    
    /// Chip decimals (usually 0 for whole chips)
    pub chip_decimals: u8,
    
    /// Token decimals (e.g., 6 for USDC, 9 for SOL)
    pub token_decimals: u8,
}

impl Default for ConversionRate {
    fn default() -> Self {
        Self {
            tokens_per_chip: 1_000_000, // 1:1 for 6 decimal token (USDC)
            chip_decimals: 0,
            token_decimals: 6,
        }
    }
}

/// Convert tokens to chips
pub fn tokens_to_chips(token_amount: u64, rate: &ConversionRate) -> u64 {
    // Convert tokens to chips based on rate
    // For 1:1 with 6 decimal token: 1_000_000 tokens = 1 chip
    token_amount / rate.tokens_per_chip
}

/// Convert chips to tokens
pub fn chips_to_tokens(chip_amount: u64, rate: &ConversionRate) -> u64 {
    // Convert chips to tokens based on rate
    chip_amount * rate.tokens_per_chip
}

/// Get conversion rate for a game
pub fn get_conversion_rate(token_decimals: u8) -> ConversionRate {
    let tokens_per_chip = 10u64.pow(token_decimals as u32);
    
    ConversionRate {
        tokens_per_chip,
        chip_decimals: 0,
        token_decimals,
    }
}

/// Calculate buy-in amount in tokens
pub fn calculate_buyin_tokens(chip_amount: u64, rate: &ConversionRate) -> u64 {
    chips_to_tokens(chip_amount, rate)
}

/// Calculate cashout amount in tokens
pub fn calculate_cashout_tokens(chip_amount: u64, rate: &ConversionRate) -> u64 {
    chips_to_tokens(chip_amount, rate)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_usdc_conversion() {
        let rate = ConversionRate {
            tokens_per_chip: 1_000_000, // 6 decimals
            chip_decimals: 0,
            token_decimals: 6,
        };
        
        // 100 USDC = 100 chips
        assert_eq!(tokens_to_chips(100_000_000, &rate), 100);
        
        // 100 chips = 100 USDC
        assert_eq!(chips_to_tokens(100, &rate), 100_000_000);
    }
    
    #[test]
    fn test_sol_conversion() {
        let rate = ConversionRate {
            tokens_per_chip: 1_000_000_000, // 9 decimals
            chip_decimals: 0,
            token_decimals: 9,
        };
        
        // 1 SOL = 1 chip
        assert_eq!(tokens_to_chips(1_000_000_000, &rate), 1);
        
        // 10 chips = 10 SOL
        assert_eq!(chips_to_tokens(10, &rate), 10_000_000_000);
    }
}
