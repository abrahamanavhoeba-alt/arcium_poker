use anchor_lang::prelude::*;
use crate::game::state::Game;
use crate::player::state::PlayerState;
use crate::cards::deck::Card;
use crate::arcium::mpc_reveal::{mpc_reveal_card, RevealParams};
use crate::betting::pot_manager::PotManager;
use crate::types::GameStage;
use crate::shared::PokerError;
use super::winner::evaluate_and_determine_winners;
use super::payout::distribute_winnings;

/// Handle showdown - reveal cards and determine winners
pub fn handle_showdown(
    game: &mut Game,
    player_states: &mut [PlayerState],
    pot_manager: &PotManager,
) -> Result<()> {
    // Validate game is in showdown stage
    require!(
        game.stage == GameStage::Showdown,
        PokerError::InvalidGameStage
    );
    
    msg!("[SHOWDOWN] Starting showdown for game {}", game.game_id);
    
    // Collect player hole cards (would be revealed via Arcium MPC)
    // For MVP, we assume cards are already decrypted
    let mut player_hole_cards = Vec::new();
    
    for i in 0..game.player_count as usize {
        if !game.active_players[i] {
            continue;
        }
        
        let player_state = &player_states[i];
        
        // Skip folded players
        if player_state.has_folded {
            continue;
        }
        
        // Use Arcium MPC to reveal encrypted hole cards
        let hole_cards = reveal_player_cards(
            player_state,
            game.shuffle_session_id,
            player_state.player,
        )?;
        
        player_hole_cards.push((i as u8, hole_cards));
    }
    
    // Get community cards
    let mut community_cards = [Card::from_index(0)?; 5];
    for i in 0..5 {
        community_cards[i] = Card::from_index(game.community_cards[i])?;
    }
    
    // Evaluate hands and determine winners
    let winners = evaluate_and_determine_winners(
        &player_hole_cards,
        &community_cards,
        pot_manager.main_pot,
        &pot_manager.side_pots,
        pot_manager.side_pot_count,
    )?;
    
    // Distribute winnings
    distribute_winnings(game, player_states, &winners)?;
    
    // Move to finished state
    game.stage = GameStage::Finished;
    
    msg!("[SHOWDOWN] Showdown complete");
    
    Ok(())
}

/// Reveal player's hole cards using Arcium MPC
pub fn reveal_player_cards(
    player_state: &PlayerState,
    session_id: [u8; 32],
    requester: Pubkey,
) -> Result<[Card; 2]> {
    let mut revealed_cards = [Card::from_index(0)?; 2];
    
    for i in 0..2 {
        // Create encrypted card from player state
        let encrypted_card = crate::arcium::mpc_deal::EncryptedCard {
            encrypted_index: player_state.encrypted_hole_cards[i],
            key_shard: [0; 32], // Would be stored separately
            owner: player_state.player,
        };
        
        // Reveal using Arcium MPC
        let reveal_params = RevealParams {
            encrypted_card,
            requester,
            session_id,
            is_showdown: true,
        };
        
        revealed_cards[i] = mpc_reveal_card(reveal_params)?;
    }
    
    Ok(revealed_cards)
}

/// Allow player to muck (fold without showing)
pub fn handle_muck(
    player_state: &mut PlayerState,
) -> Result<()> {
    // Player folds without revealing cards
    player_state.fold();
    
    msg!(
        "[SHOWDOWN] Player {} mucked their hand",
        player_state.player
    );
    
    Ok(())
}