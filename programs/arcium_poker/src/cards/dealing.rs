use anchor_lang::prelude::*;
use crate::game::state::Game;
use crate::player::state::PlayerState;
use crate::arcium::mpc_deal::{mpc_deal_card, DealParams, EncryptedCard};
use crate::shared::{constants::*, PokerError};
use crate::types::GameStage;

/// Deal hole cards to all players
pub fn deal_hole_cards(
    game: &mut Game,
    player_states: &mut [Account<PlayerState>],
) -> Result<()> {
    require!(
        game.stage == GameStage::PreFlop,
        PokerError::InvalidGameStage
    );
    require!(game.deck_initialized, PokerError::DeckNotInitialized);
    
    msg!("[DEALING] Dealing {} hole cards to {} players", HOLE_CARDS, game.player_count);
    
    // Deal HOLE_CARDS (2) cards to each player
    for player_state in player_states.iter_mut() {
        if !player_state.has_cards {
            deal_cards_to_player(game, player_state)?;
        }
    }
    
    msg!("[DEALING] All hole cards dealt");
    Ok(())
}

/// Deal encrypted cards to a specific player
fn deal_cards_to_player(
    game: &mut Game,
    player_state: &mut PlayerState,
) -> Result<()> {
    msg!("[DEALING] Dealing to player at seat {}", player_state.seat_index);
    
    // Deal hole cards using Arcium MPC
    for i in 0..HOLE_CARDS {
        // Get next encrypted card index from deck
        let mut encrypted_deck = game.get_encrypted_deck()?;
        let card_index = encrypted_deck.get_next_encrypted_card()?;
        
        // Use Arcium MPC to deal encrypted card to player
        let deal_params = DealParams {
            card_index,
            player: player_state.player,
            session_id: game.encrypted_deck,
            game_id: game.game_id,
        };
        
        let encrypted_card = mpc_deal_card(deal_params)?;
        
        // Store encrypted card in player state
        player_state.encrypted_hole_cards[i] = encrypted_card.encrypted_index;
        
        msg!(
            "[DEALING] Card {} dealt to seat {} (encrypted: {})",
            i + 1,
            player_state.seat_index,
            encrypted_card.encrypted_index
        );
    }
    
    player_state.has_cards = true;
    Ok(())
}

/// Reveal community cards (flop/turn/river)
pub fn reveal_community_cards(
    game: &mut Game,
    count: u8,
) -> Result<()> {
    require!(game.deck_initialized, PokerError::DeckNotInitialized);
    
    msg!("[DEALING] Revealing {} community cards", count);
    
    // Burn a card first (poker rules)
    let mut encrypted_deck = game.get_encrypted_deck()?;
    encrypted_deck.burn_card()?;
    msg!("[DEALING] Burn card dealt");
    
    // Reveal community cards
    for i in 0..count {
        let card_index = encrypted_deck.get_next_encrypted_card()?;
        let community_index = game.community_cards_revealed as usize;
        
        // Store card index in community cards array
        game.community_cards[community_index] = card_index;
        game.community_cards_revealed += 1;
        
        msg!(
            "[DEALING] Community card {} revealed (index: {})",
            community_index + 1,
            card_index
        );
    }
    
    Ok(())
}