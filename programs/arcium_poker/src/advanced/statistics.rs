use anchor_lang::prelude::*;
use crate::cards::evaluator::EvaluatedHand;
use crate::types::HandRank;

/// Player statistics
#[account]
pub struct PlayerStats {
    /// Player public key
    pub player: Pubkey,
    
    /// Total hands played
    pub hands_played: u64,
    
    /// Hands won
    pub hands_won: u64,
    
    /// Total winnings (in chips)
    pub total_winnings: u64,
    
    /// Total losses (in chips)
    pub total_losses: u64,
    
    /// Biggest pot won
    pub biggest_pot_won: u64,
    
    /// Hands folded
    pub hands_folded: u64,
    
    /// Hands went to showdown
    pub showdowns: u64,
    
    /// Showdowns won
    pub showdowns_won: u64,
    
    /// All-ins
    pub all_ins: u64,
    
    /// All-ins won
    pub all_ins_won: u64,
    
    /// Best hand achieved
    pub best_hand_rank: u8,
    
    /// Total rake paid
    pub total_rake_paid: u64,
    
    /// Games played
    pub games_played: u64,
    
    /// Last played timestamp
    pub last_played_at: i64,
    
    /// Created timestamp
    pub created_at: i64,
    
    /// Bump seed
    pub bump: u8,
}

impl PlayerStats {
    pub const LEN: usize = 8 + // discriminator
        32 + // player
        8 + // hands_played
        8 + // hands_won
        8 + // total_winnings
        8 + // total_losses
        8 + // biggest_pot_won
        8 + // hands_folded
        8 + // showdowns
        8 + // showdowns_won
        8 + // all_ins
        8 + // all_ins_won
        1 + // best_hand_rank
        8 + // total_rake_paid
        8 + // games_played
        8 + // last_played_at
        8 + // created_at
        1; // bump
    
    /// Calculate win rate
    pub fn win_rate(&self) -> f64 {
        if self.hands_played == 0 {
            return 0.0;
        }
        (self.hands_won as f64 / self.hands_played as f64) * 100.0
    }
    
    /// Calculate net profit
    pub fn net_profit(&self) -> i64 {
        self.total_winnings as i64 - self.total_losses as i64
    }
    
    /// Calculate showdown win rate
    pub fn showdown_win_rate(&self) -> f64 {
        if self.showdowns == 0 {
            return 0.0;
        }
        (self.showdowns_won as f64 / self.showdowns as f64) * 100.0
    }
    
    /// Calculate all-in win rate
    pub fn all_in_win_rate(&self) -> f64 {
        if self.all_ins == 0 {
            return 0.0;
        }
        (self.all_ins_won as f64 / self.all_ins as f64) * 100.0
    }
}

/// Initialize player statistics
pub fn initialize_player_stats(
    stats: &mut PlayerStats,
    player: Pubkey,
    bump: u8,
) -> Result<()> {
    stats.player = player;
    stats.hands_played = 0;
    stats.hands_won = 0;
    stats.total_winnings = 0;
    stats.total_losses = 0;
    stats.biggest_pot_won = 0;
    stats.hands_folded = 0;
    stats.showdowns = 0;
    stats.showdowns_won = 0;
    stats.all_ins = 0;
    stats.all_ins_won = 0;
    stats.best_hand_rank = 0;
    stats.total_rake_paid = 0;
    stats.games_played = 0;
    stats.last_played_at = Clock::get()?.unix_timestamp;
    stats.created_at = Clock::get()?.unix_timestamp;
    stats.bump = bump;
    
    msg!("[STATS] Initialized stats for player {}", player);
    
    Ok(())
}

/// Update hand played
pub fn update_hand_played(stats: &mut PlayerStats) -> Result<()> {
    stats.hands_played += 1;
    stats.last_played_at = Clock::get()?.unix_timestamp;
    
    Ok(())
}

/// Update win statistics
pub fn update_win_stats(
    stats: &mut PlayerStats,
    pot_won: u64,
    went_to_showdown: bool,
) -> Result<()> {
    stats.hands_won += 1;
    stats.total_winnings += pot_won;
    
    if pot_won > stats.biggest_pot_won {
        stats.biggest_pot_won = pot_won;
    }
    
    if went_to_showdown {
        stats.showdowns_won += 1;
    }
    
    msg!(
        "[STATS] Player {} won {} chips. Total wins: {}",
        stats.player,
        pot_won,
        stats.hands_won
    );
    
    Ok(())
}

/// Record pot won
pub fn record_pot_won(
    stats: &mut PlayerStats,
    amount: u64,
) -> Result<()> {
    stats.total_winnings += amount;
    
    if amount > stats.biggest_pot_won {
        stats.biggest_pot_won = amount;
        msg!(
            "[STATS] New biggest pot for {}: {}",
            stats.player,
            amount
        );
    }
    
    Ok(())
}

/// Update fold statistics
pub fn update_fold_stats(stats: &mut PlayerStats) -> Result<()> {
    stats.hands_folded += 1;
    
    Ok(())
}

/// Update showdown statistics
pub fn update_showdown_stats(
    stats: &mut PlayerStats,
    won: bool,
) -> Result<()> {
    stats.showdowns += 1;
    
    if won {
        stats.showdowns_won += 1;
    }
    
    Ok(())
}

/// Update all-in statistics
pub fn update_all_in_stats(
    stats: &mut PlayerStats,
    won: bool,
) -> Result<()> {
    stats.all_ins += 1;
    
    if won {
        stats.all_ins_won += 1;
    }
    
    Ok(())
}

/// Update best hand
pub fn update_best_hand(
    stats: &mut PlayerStats,
    hand: &EvaluatedHand,
) -> Result<()> {
    let hand_rank = hand.rank as u8;
    
    if hand_rank > stats.best_hand_rank {
        stats.best_hand_rank = hand_rank;
        
        msg!(
            "[STATS] New best hand for {}: {:?}",
            stats.player,
            hand.rank
        );
    }
    
    Ok(())
}

/// Update rake paid
pub fn update_rake_paid(
    stats: &mut PlayerStats,
    rake_amount: u64,
) -> Result<()> {
    stats.total_rake_paid += rake_amount;
    
    Ok(())
}

/// Update game played
pub fn update_game_played(stats: &mut PlayerStats) -> Result<()> {
    stats.games_played += 1;
    
    Ok(())
}

/// Get player stats summary
pub fn get_player_stats(stats: &PlayerStats) -> PlayerStatsSummary {
    PlayerStatsSummary {
        player: stats.player,
        hands_played: stats.hands_played,
        hands_won: stats.hands_won,
        win_rate: stats.win_rate(),
        total_winnings: stats.total_winnings,
        total_losses: stats.total_losses,
        net_profit: stats.net_profit(),
        biggest_pot_won: stats.biggest_pot_won,
        showdown_win_rate: stats.showdown_win_rate(),
        all_in_win_rate: stats.all_in_win_rate(),
        best_hand_rank: stats.best_hand_rank,
    }
}

/// Player statistics summary
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PlayerStatsSummary {
    pub player: Pubkey,
    pub hands_played: u64,
    pub hands_won: u64,
    pub win_rate: f64,
    pub total_winnings: u64,
    pub total_losses: u64,
    pub net_profit: i64,
    pub biggest_pot_won: u64,
    pub showdown_win_rate: f64,
    pub all_in_win_rate: f64,
    pub best_hand_rank: u8,
}

/// Leaderboard entry
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LeaderboardEntry {
    pub player: Pubkey,
    pub total_winnings: u64,
    pub hands_won: u64,
    pub win_rate: f64,
}

/// Get leaderboard from multiple player stats
pub fn create_leaderboard(
    all_stats: &[PlayerStats],
    limit: usize,
) -> Vec<LeaderboardEntry> {
    let mut entries: Vec<LeaderboardEntry> = all_stats
        .iter()
        .map(|stats| LeaderboardEntry {
            player: stats.player,
            total_winnings: stats.total_winnings,
            hands_won: stats.hands_won,
            win_rate: stats.win_rate(),
        })
        .collect();
    
    // Sort by total winnings
    entries.sort_by(|a, b| b.total_winnings.cmp(&a.total_winnings));
    
    // Take top N
    entries.truncate(limit);
    
    entries
}
