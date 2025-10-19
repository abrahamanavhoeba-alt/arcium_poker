use anchor_lang::prelude::*;
use crate::cards::evaluator::{EvaluatedHand, evaluate_best_hand};
use crate::cards::deck::Card;
use crate::betting::state::SidePot;
use crate::shared::constants::MAX_PLAYERS;
use crate::shared::PokerError;

/// Winner information for a pot
#[derive(Clone, Debug)]
pub struct PotWinner {
    pub seat_index: u8,
    pub hand: EvaluatedHand,
    pub share: u64,  // Amount won from this pot
}

/// Determine winners for main pot
pub fn determine_main_pot_winners(
    player_hands: &[(u8, EvaluatedHand)], // (seat_index, hand)
    pot_amount: u64,
) -> Vec<PotWinner> {
    if player_hands.is_empty() {
        return Vec::new();
    }
    
    // Find best hand
    let best_hand = player_hands
        .iter()
        .map(|(_, hand)| hand)
        .max()
        .unwrap();
    
    // Find all players with best hand (for splits)
    let winners: Vec<&(u8, EvaluatedHand)> = player_hands
        .iter()
        .filter(|(_, hand)| hand == best_hand)
        .collect();
    
    // Split pot among winners
    let share = pot_amount / winners.len() as u64;
    let remainder = pot_amount % winners.len() as u64;
    
    winners
        .into_iter()
        .enumerate()
        .map(|(i, (seat, hand))| PotWinner {
            seat_index: *seat,
            hand: *hand,
            share: if i == 0 { share + remainder } else { share },
        })
        .collect()
}

/// Determine winners for a side pot
pub fn determine_side_pot_winners(
    player_hands: &[(u8, EvaluatedHand)],
    side_pot: &SidePot,
) -> Vec<PotWinner> {
    // Filter to only eligible players
    let eligible_hands: Vec<(u8, EvaluatedHand)> = player_hands
        .iter()
        .filter(|(seat, _)| side_pot.is_eligible(*seat as usize))
        .copied()
        .collect();
    
    determine_main_pot_winners(&eligible_hands, side_pot.amount)
}

/// Determine all winners (main pot + side pots)
pub fn determine_all_winners(
    player_hands: &[(u8, EvaluatedHand)],
    main_pot: u64,
    side_pots: &[SidePot],
    side_pot_count: u8,
) -> Vec<(u8, u64)> { // Returns (seat_index, total_winnings)
    let mut total_winnings = [0u64; MAX_PLAYERS];
    
    // Determine side pot winners first (from smallest to largest)
    for i in 0..side_pot_count as usize {
        let winners = determine_side_pot_winners(player_hands, &side_pots[i]);
        for winner in winners {
            total_winnings[winner.seat_index as usize] += winner.share;
        }
    }
    
    // Determine main pot winners
    let main_winners = determine_main_pot_winners(player_hands, main_pot);
    for winner in main_winners {
        total_winnings[winner.seat_index as usize] += winner.share;
    }
    
    // Convert to vec of (seat, winnings) for non-zero amounts
    total_winnings
        .iter()
        .enumerate()
        .filter(|(_, &amount)| amount > 0)
        .map(|(seat, &amount)| (seat as u8, amount))
        .collect()
}

/// Evaluate all player hands and determine winners
pub fn evaluate_and_determine_winners(
    player_hole_cards: &[(u8, [Card; 2])], // (seat_index, hole_cards)
    community_cards: &[Card; 5],
    main_pot: u64,
    side_pots: &[SidePot],
    side_pot_count: u8,
) -> Result<Vec<(u8, u64)>> {
    // Evaluate all hands
    let mut evaluated_hands = Vec::new();
    
    for (seat, hole_cards) in player_hole_cards {
        let hand = evaluate_best_hand(hole_cards, community_cards)?;
        evaluated_hands.push((*seat, hand));
        
        msg!(
            "[SHOWDOWN] Seat {} hand: {:?} (primary: {}, secondary: {})",
            seat,
            hand.rank,
            hand.primary_value,
            hand.secondary_value
        );
    }
    
    // Determine winners
    let winners = determine_all_winners(
        &evaluated_hands,
        main_pot,
        side_pots,
        side_pot_count,
    );
    
    // Log winners
    for (seat, amount) in &winners {
        msg!("[SHOWDOWN] Seat {} wins {}", seat, amount);
    }
    
    Ok(winners)
}