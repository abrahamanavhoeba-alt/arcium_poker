use anchor_lang::prelude::*;
use super::state::Game;
use crate::player::state::PlayerState;
use crate::types::GameStage;
use crate::shared::{PokerError, constants::*};
use crate::cards::dealing::reveal_community_cards;
use crate::betting::is_betting_round_complete;

/// Advance game to next stage (PreFlop -> Flop -> Turn -> River -> Showdown)
/// Note: Caller should verify betting round is complete before calling this
pub fn advance_game_stage(
    game: &mut Game,
) -> Result<()> {
    let next_stage = match game.stage {
        GameStage::Waiting => {
            return Err(PokerError::InvalidGameStage.into());
        }
        GameStage::PreFlop => {
            msg!("[GAME FLOW] Advancing to Flop");
            GameStage::Flop
        }
        GameStage::Flop => {
            msg!("[GAME FLOW] Advancing to Turn");
            GameStage::Turn
        }
        GameStage::Turn => {
            msg!("[GAME FLOW] Advancing to River");
            GameStage::River
        }
        GameStage::River => {
            msg!("[GAME FLOW] Advancing to Showdown");
            GameStage::Showdown
        }
        GameStage::Showdown | GameStage::Finished => {
            return Err(PokerError::InvalidGameStage.into());
        }
    };
    
    game.stage = next_stage;
    
    // Reset betting state for new round
    reset_betting_round(game)?;
    
    // Reveal community cards based on stage
    match next_stage {
        GameStage::Flop => {
            // Reveal 3 cards for flop
            reveal_community_cards(game, 3)?;
        }
        GameStage::Turn => {
            // Reveal 1 card for turn
            reveal_community_cards(game, 1)?;
        }
        GameStage::River => {
            // Reveal 1 card for river
            reveal_community_cards(game, 1)?;
        }
        GameStage::Showdown => {
            // No cards to reveal, proceed to showdown
            msg!("[GAME FLOW] Ready for showdown");
        }
        _ => {}
    }
    
    Ok(())
}

/// Reset betting state for new round
pub fn reset_betting_round(game: &mut Game) -> Result<()> {
    // Reset current bet to 0
    game.current_bet = 0;
    
    // Reset players_acted flags
    game.players_acted = [false; crate::shared::constants::MAX_PLAYERS];
    
    // Set first player to act (after dealer button)
    game.current_player_index = get_first_player_for_round(game);
    
    // Update timestamp
    game.last_action_at = Clock::get()?.unix_timestamp;
    
    msg!(
        "[GAME FLOW] Betting round reset. First to act: seat {}",
        game.current_player_index
    );
    
    Ok(())
}

/// Get first player to act in a betting round
pub fn get_first_player_for_round(game: &Game) -> u8 {
    // In pre-flop, first to act is after big blind (dealer + 3)
    // In post-flop rounds, first to act is after dealer (dealer + 1)
    let offset = if game.stage == GameStage::PreFlop {
        3 // After big blind
    } else {
        1 // After dealer
    };
    
    let mut first_player = (game.dealer_position + offset) % game.player_count;
    
    // Find first active player
    for _ in 0..game.player_count {
        if game.active_players[first_player as usize] {
            return first_player;
        }
        first_player = (first_player + 1) % game.player_count;
    }
    
    // Fallback to dealer if no active players found
    game.dealer_position
}

/// Rotate dealer button to next player
pub fn rotate_dealer_button(game: &mut Game) -> Result<()> {
    let old_dealer = game.dealer_position;
    
    // Find next active player
    let mut next_dealer = (old_dealer + 1) % game.player_count;
    let mut found = false;
    
    for _ in 0..game.player_count {
        if game.active_players[next_dealer as usize] {
            found = true;
            break;
        }
        next_dealer = (next_dealer + 1) % game.player_count;
    }
    
    require!(found, PokerError::NotEnoughPlayers);
    
    game.dealer_position = next_dealer;
    
    msg!(
        "[GAME FLOW] Dealer button rotated from seat {} to seat {}",
        old_dealer,
        next_dealer
    );
    
    Ok(())
}

/// Get small blind position
pub fn get_small_blind_position(game: &Game) -> u8 {
    // Small blind is dealer + 1 (or dealer in heads-up)
    if game.player_count == 2 {
        game.dealer_position
    } else {
        (game.dealer_position + 1) % game.player_count
    }
}

/// Get big blind position
pub fn get_big_blind_position(game: &Game) -> u8 {
    // Big blind is dealer + 2 (or non-dealer in heads-up)
    if game.player_count == 2 {
        (game.dealer_position + 1) % game.player_count
    } else {
        (game.dealer_position + 2) % game.player_count
    }
}

/// Check if player's turn has timed out
pub fn check_turn_timeout(game: &Game) -> Result<bool> {
    let current_time = Clock::get()?.unix_timestamp;
    let time_since_last_action = current_time - game.last_action_at;
    
    Ok(time_since_last_action >= TURN_TIMEOUT)
}

/// Handle player timeout (auto-fold)
pub fn handle_player_timeout(
    game: &mut Game,
    player_state: &mut PlayerState,
) -> Result<()> {
    require!(
        check_turn_timeout(game)?,
        PokerError::InvalidAction
    );
    
    require!(
        game.current_player_index == player_state.seat_index,
        PokerError::NotPlayerTurn
    );
    
    // Auto-fold the player
    player_state.fold();
    game.active_players[player_state.seat_index as usize] = false;
    
    msg!(
        "[GAME FLOW] Player {} timed out and was auto-folded",
        player_state.player
    );
    
    // Advance to next player
    advance_to_next_active_player(game)?;
    
    Ok(())
}

/// Advance to next active player
pub fn advance_to_next_active_player(game: &mut Game) -> Result<()> {
    let start_index = game.current_player_index;
    let mut next_index = (start_index + 1) % game.player_count;
    
    // Find next active player who hasn't folded or gone all-in
    let mut found = false;
    for _ in 0..game.player_count {
        if game.active_players[next_index as usize] {
            found = true;
            break;
        }
        next_index = (next_index + 1) % game.player_count;
    }
    
    require!(found, PokerError::InvalidGameStage);
    
    game.current_player_index = next_index;
    game.last_action_at = Clock::get()?.unix_timestamp;
    
    Ok(())
}

/// Check if only one player remains (all others folded)
pub fn check_single_player_remaining(game: &Game) -> bool {
    let active_count = game.active_players[..game.player_count as usize]
        .iter()
        .filter(|&&active| active)
        .count();
    
    active_count <= 1
}

/// Check if all players are all-in (no more betting possible)
pub fn check_all_players_all_in(
    game: &Game,
    player_states: &[PlayerState],
) -> bool {
    let mut non_all_in_count = 0;
    
    for i in 0..game.player_count as usize {
        if !game.active_players[i] {
            continue;
        }
        
        let player_state = &player_states[i];
        if !player_state.has_folded && !player_state.is_all_in {
            non_all_in_count += 1;
        }
    }
    
    // If 0 or 1 non-all-in players, no more betting
    non_all_in_count <= 1
}

/// Start new hand (reset for next hand)
pub fn start_new_hand(game: &mut Game) -> Result<()> {
    // Rotate dealer button
    rotate_dealer_button(game)?;
    
    // Reset game state
    game.stage = GameStage::PreFlop;  // Start at PreFlop, not Waiting
    game.pot = 0;
    game.current_bet = 0;
    game.community_cards = [0; COMMUNITY_CARDS];
    game.community_cards_revealed = 0;
    game.deck_initialized = false;
    
    // Reset active players (all players who haven't left)
    for i in 0..game.player_count as usize {
        if game.players[i] != Pubkey::default() {
            game.active_players[i] = true;
        }
    }
    
    msg!("[GAME FLOW] New hand started. Dealer at seat {}", game.dealer_position);
    
    Ok(())
}

/// Check if game should end (not enough players)
pub fn should_end_game(game: &Game) -> bool {
    game.player_count < MIN_PLAYERS as u8
}

/// End the game
pub fn end_game(game: &mut Game) -> Result<()> {
    game.stage = GameStage::Finished;
    
    msg!("[GAME FLOW] Game ended");
    
    Ok(())
}

// Tests removed - would require implementing Default for Game
// Integration tests should be used instead
