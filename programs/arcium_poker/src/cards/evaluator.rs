use anchor_lang::prelude::*;
use super::deck::Card;
use crate::types::{HandRank, Rank, Suit};
use crate::shared::PokerError;

/// Evaluated hand with rank and kickers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvaluatedHand {
    pub rank: HandRank,
    pub primary_value: u8,    // Main card value (e.g., pair value, three of a kind value)
    pub secondary_value: u8,  // Secondary value (e.g., second pair in two pair)
    pub kickers: [u8; 5],     // Kicker cards for tie-breaking
}

impl EvaluatedHand {
    pub fn new(rank: HandRank, primary: u8, secondary: u8, kickers: [u8; 5]) -> Self {
        Self {
            rank,
            primary_value: primary,
            secondary_value: secondary,
            kickers,
        }
    }
}

impl PartialOrd for EvaluatedHand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EvaluatedHand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare hand rank first
        match self.rank.cmp(&other.rank) {
            std::cmp::Ordering::Equal => {
                // Same rank, compare primary value
                match self.primary_value.cmp(&other.primary_value) {
                    std::cmp::Ordering::Equal => {
                        // Same primary, compare secondary
                        match self.secondary_value.cmp(&other.secondary_value) {
                            std::cmp::Ordering::Equal => {
                                // Compare kickers
                                self.kickers.cmp(&other.kickers)
                            }
                            other => other,
                        }
                    }
                    other => other,
                }
            }
            other => other,
        }
    }
}

/// Evaluate a 5-card poker hand
pub fn evaluate_hand(cards: &[Card; 5]) -> Result<EvaluatedHand> {
    require!(cards.len() == 5, PokerError::InvalidCardIndex);
    
    // Check for flush
    let is_flush = is_flush(cards);
    
    // Check for straight
    let straight_high = check_straight(cards);
    let is_straight = straight_high.is_some();
    
    // Get rank counts
    let rank_counts = count_ranks(cards);
    
    // Check for straight flush / royal flush
    if is_flush && is_straight {
        let high = straight_high.unwrap();
        if high == 14 { // Ace high straight flush
            return Ok(EvaluatedHand::new(HandRank::RoyalFlush, 14, 0, [14, 13, 12, 11, 10]));
        } else {
            return Ok(EvaluatedHand::new(HandRank::StraightFlush, high, 0, get_kickers(cards, &[])));
        }
    }
    
    // Check for four of a kind
    if let Some((quad_rank, kicker)) = check_four_of_kind(&rank_counts) {
        return Ok(EvaluatedHand::new(HandRank::FourOfAKind, quad_rank, 0, [kicker, 0, 0, 0, 0]));
    }
    
    // Check for full house
    if let Some((trips_rank, pair_rank)) = check_full_house(&rank_counts) {
        return Ok(EvaluatedHand::new(HandRank::FullHouse, trips_rank, pair_rank, [0; 5]));
    }
    
    // Check for flush
    if is_flush {
        let kickers = get_kickers(cards, &[]);
        return Ok(EvaluatedHand::new(HandRank::Flush, kickers[0], 0, kickers));
    }
    
    // Check for straight
    if is_straight {
        let high = straight_high.unwrap();
        return Ok(EvaluatedHand::new(HandRank::Straight, high, 0, [0; 5]));
    }
    
    // Check for three of a kind
    if let Some((trips_rank, kickers)) = check_three_of_kind(&rank_counts, cards) {
        return Ok(EvaluatedHand::new(HandRank::ThreeOfAKind, trips_rank, 0, kickers));
    }
    
    // Check for two pair
    if let Some((high_pair, low_pair, kicker)) = check_two_pair(&rank_counts) {
        return Ok(EvaluatedHand::new(HandRank::TwoPair, high_pair, low_pair, [kicker, 0, 0, 0, 0]));
    }
    
    // Check for one pair
    if let Some((pair_rank, kickers)) = check_one_pair(&rank_counts, cards) {
        return Ok(EvaluatedHand::new(HandRank::OnePair, pair_rank, 0, kickers));
    }
    
    // High card
    let kickers = get_kickers(cards, &[]);
    Ok(EvaluatedHand::new(HandRank::HighCard, kickers[0], 0, kickers))
}

/// Evaluate best 5-card hand from 7 cards (2 hole + 5 community)
pub fn evaluate_best_hand(hole_cards: &[Card; 2], community_cards: &[Card; 5]) -> Result<EvaluatedHand> {
    let mut all_cards = Vec::with_capacity(7);
    all_cards.extend_from_slice(hole_cards);
    all_cards.extend_from_slice(community_cards);
    
    // Try all combinations of 5 cards from 7
    let mut best_hand: Option<EvaluatedHand> = None;
    
    // Generate all 5-card combinations from 7 cards (21 combinations)
    for i in 0..7 {
        for j in (i+1)..7 {
            // Skip cards i and j, use the other 5
            let mut hand = [all_cards[0]; 5];
            let mut idx = 0;
            for k in 0..7 {
                if k != i && k != j {
                    hand[idx] = all_cards[k];
                    idx += 1;
                }
            }
            
            let evaluated = evaluate_hand(&hand)?;
            
            if best_hand.is_none() || evaluated > best_hand.unwrap() {
                best_hand = Some(evaluated);
            }
        }
    }
    
    best_hand.ok_or(PokerError::InvalidCardIndex.into())
}

/// Check if all cards are same suit
fn is_flush(cards: &[Card]) -> bool {
    let first_suit = cards[0].suit;
    cards.iter().all(|c| c.suit == first_suit)
}

/// Check for straight, returns high card if straight
fn check_straight(cards: &[Card]) -> Option<u8> {
    let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank as u8).collect();
    ranks.sort_unstable();
    ranks.reverse();
    
    // Check for regular straight
    if ranks[0] - ranks[4] == 4 && 
       ranks[0] - ranks[1] == 1 &&
       ranks[1] - ranks[2] == 1 &&
       ranks[2] - ranks[3] == 1 &&
       ranks[3] - ranks[4] == 1 {
        return Some(ranks[0]);
    }
    
    // Check for wheel (A-2-3-4-5)
    if ranks[0] == 14 && ranks[1] == 5 && ranks[2] == 4 && ranks[3] == 3 && ranks[4] == 2 {
        return Some(5); // 5-high straight
    }
    
    None
}

/// Count occurrences of each rank
fn count_ranks(cards: &[Card]) -> [u8; 15] {
    let mut counts = [0u8; 15];
    for card in cards {
        counts[card.rank as usize] += 1;
    }
    counts
}

/// Check for four of a kind
fn check_four_of_kind(rank_counts: &[u8; 15]) -> Option<(u8, u8)> {
    let mut quad_rank = 0;
    let mut kicker = 0;
    
    for (rank, &count) in rank_counts.iter().enumerate() {
        if count == 4 {
            quad_rank = rank as u8;
        } else if count == 1 {
            kicker = rank as u8;
        }
    }
    
    if quad_rank > 0 {
        Some((quad_rank, kicker))
    } else {
        None
    }
}

/// Check for full house
fn check_full_house(rank_counts: &[u8; 15]) -> Option<(u8, u8)> {
    let mut trips_rank = 0;
    let mut pair_rank = 0;
    
    for (rank, &count) in rank_counts.iter().enumerate().rev() {
        if count == 3 && trips_rank == 0 {
            trips_rank = rank as u8;
        } else if count == 2 && pair_rank == 0 {
            pair_rank = rank as u8;
        } else if count == 3 && pair_rank == 0 {
            pair_rank = rank as u8;
        }
    }
    
    if trips_rank > 0 && pair_rank > 0 {
        Some((trips_rank, pair_rank))
    } else {
        None
    }
}

/// Check for three of a kind
fn check_three_of_kind(rank_counts: &[u8; 15], cards: &[Card]) -> Option<(u8, [u8; 5])> {
    let mut trips_rank = 0;
    
    for (rank, &count) in rank_counts.iter().enumerate() {
        if count == 3 {
            trips_rank = rank as u8;
            break;
        }
    }
    
    if trips_rank > 0 {
        let kickers = get_kickers(cards, &[trips_rank]);
        Some((trips_rank, kickers))
    } else {
        None
    }
}

/// Check for two pair
fn check_two_pair(rank_counts: &[u8; 15]) -> Option<(u8, u8, u8)> {
    let mut pairs = Vec::new();
    let mut kicker = 0;
    
    for (rank, &count) in rank_counts.iter().enumerate().rev() {
        if count == 2 {
            pairs.push(rank as u8);
        } else if count == 1 {
            kicker = rank as u8;
        }
    }
    
    if pairs.len() >= 2 {
        Some((pairs[0], pairs[1], kicker))
    } else {
        None
    }
}

/// Check for one pair
fn check_one_pair(rank_counts: &[u8; 15], cards: &[Card]) -> Option<(u8, [u8; 5])> {
    let mut pair_rank = 0;
    
    for (rank, &count) in rank_counts.iter().enumerate().rev() {
        if count == 2 {
            pair_rank = rank as u8;
            break;
        }
    }
    
    if pair_rank > 0 {
        let kickers = get_kickers(cards, &[pair_rank]);
        Some((pair_rank, kickers))
    } else {
        None
    }
}

/// Get kicker cards (excluding specified ranks)
fn get_kickers(cards: &[Card], exclude_ranks: &[u8]) -> [u8; 5] {
    let mut ranks: Vec<u8> = cards
        .iter()
        .map(|c| c.rank as u8)
        .filter(|r| !exclude_ranks.contains(r))
        .collect();
    
    ranks.sort_unstable();
    ranks.reverse();
    
    let mut kickers = [0u8; 5];
    for (i, &rank) in ranks.iter().take(5).enumerate() {
        kickers[i] = rank;
    }
    
    kickers
}