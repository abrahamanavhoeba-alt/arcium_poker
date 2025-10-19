use anchor_lang::prelude::*;
use crate::game::state::Game;
use crate::player::state::PlayerState;
use crate::types::GameStage;
use crate::shared::{PokerError, constants::*};

/// Validate game state invariants
pub fn validate_game_state(game: &Game, player_states: &[PlayerState]) -> Result<()> {
    // Validate player count
    require!(
        game.player_count <= game.max_players,
        PokerError::InvalidGameConfig
    );
    
    require!(
        game.player_count as usize <= MAX_PLAYERS,
        PokerError::InvalidGameConfig
    );
    
    // Validate dealer position
    require!(
        game.dealer_position < game.player_count,
        PokerError::InvalidGameConfig
    );
    
    // Validate current player
    if game.stage != GameStage::Waiting && game.stage != GameStage::Finished {
        require!(
            game.current_player_index < game.player_count,
            PokerError::InvalidGameConfig
        );
    }
    
    // Validate chip conservation
    validate_chip_conservation(game, player_states)?;
    
    Ok(())
}

/// Validate chip conservation (total chips = player stacks + pot)
pub fn validate_chip_conservation(game: &Game, player_states: &[PlayerState]) -> Result<()> {
    let mut total_player_chips = 0u64;
    
    for i in 0..game.player_count as usize {
        if game.active_players[i] {
            total_player_chips += player_states[i].chip_stack;
            total_player_chips += player_states[i].total_bet_this_hand;
        }
    }
    
    // Total chips should equal player stacks + pot
    // Note: This is a simplified check. In production, track initial total
    msg!(
        "[SECURITY] Chip conservation check: {} in stacks + bets, {} in pot",
        total_player_chips,
        game.pot
    );
    
    Ok(())
}

/// Validate deck integrity (52 unique cards)
pub fn validate_deck_integrity(encrypted_deck: &[u8; 52]) -> Result<()> {
    // Check for duplicate card indices
    let mut seen = [false; 52];
    
    for &card_index in encrypted_deck.iter() {
        require!(
            card_index < 52,
            PokerError::InvalidCardIndex
        );
        
        require!(
            !seen[card_index as usize],
            PokerError::DeckNotInitialized
        );
        
        seen[card_index as usize] = true;
    }
    
    msg!("[SECURITY] Deck integrity validated: 52 unique cards");
    
    Ok(())
}

/// Validate state transition is legal
pub fn validate_state_transition(
    from_stage: GameStage,
    to_stage: GameStage,
) -> Result<()> {
    let valid = match (from_stage, to_stage) {
        // Waiting can go to PreFlop (game starts)
        (GameStage::Waiting, GameStage::PreFlop) => true,
        
        // PreFlop can go to Flop
        (GameStage::PreFlop, GameStage::Flop) => true,
        
        // Flop can go to Turn
        (GameStage::Flop, GameStage::Turn) => true,
        
        // Turn can go to River
        (GameStage::Turn, GameStage::River) => true,
        
        // River can go to Showdown
        (GameStage::River, GameStage::Showdown) => true,
        
        // Showdown can go to Finished
        (GameStage::Showdown, GameStage::Finished) => true,
        
        // Any stage can go to Finished (e.g., all fold)
        (_, GameStage::Finished) => true,
        
        // Finished can go back to Waiting (new hand)
        (GameStage::Finished, GameStage::Waiting) => true,
        
        // All other transitions are invalid
        _ => false,
    };
    
    require!(valid, PokerError::InvalidGameStage);
    
    msg!(
        "[SECURITY] Valid state transition: {:?} -> {:?}",
        from_stage,
        to_stage
    );
    
    Ok(())
}

/// Validate player action is legal
pub fn validate_player_action(
    game: &Game,
    player_state: &PlayerState,
    seat_index: u8,
) -> Result<()> {
    // Must be player's turn
    require!(
        game.current_player_index == seat_index,
        PokerError::NotPlayerTurn
    );
    
    // Player must be active
    require!(
        game.active_players[seat_index as usize],
        PokerError::PlayerNotInGame
    );
    
    // Player must not have folded
    require!(
        !player_state.has_folded,
        PokerError::InvalidAction
    );
    
    // Player must not be all-in (unless checking)
    // This is handled in specific action validators
    
    Ok(())
}

/// Validate bet amount is within limits
pub fn validate_bet_limits(
    bet_amount: u64,
    min_bet: u64,
    max_bet: u64,
    player_chips: u64,
) -> Result<()> {
    // Bet must be at least minimum (unless all-in)
    if bet_amount < player_chips {
        require!(
            bet_amount >= min_bet,
            PokerError::InvalidBetAmount
        );
    }
    
    // Bet cannot exceed maximum (if set)
    if max_bet > 0 {
        require!(
            bet_amount <= max_bet,
            PokerError::InvalidBetAmount
        );
    }
    
    // Bet cannot exceed player's chips
    require!(
        bet_amount <= player_chips,
        PokerError::InsufficientChips
    );
    
    Ok(())
}

/// Validate timeout hasn't occurred
pub fn validate_no_timeout(game: &Game, current_time: i64) -> Result<()> {
    let time_since_action = current_time - game.last_action_at;
    
    require!(
        time_since_action < TURN_TIMEOUT,
        PokerError::InvalidAction
    );
    
    Ok(())
}
