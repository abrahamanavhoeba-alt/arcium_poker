use anchor_lang::prelude::*;
use crate::game::state::Game;
use crate::player::state::PlayerState;
use crate::types::{GameStage, PlayerAction};
use crate::shared::{PokerError, constants::*};

/// Validate that it's the player's turn
pub fn validate_player_turn(
    game: &Game,
    player_seat: u8,
) -> Result<()> {
    require!(
        game.current_player_index == player_seat,
        PokerError::NotPlayerTurn
    );
    Ok(())
}

/// Validate player has sufficient chips for action
pub fn validate_sufficient_chips(
    player_state: &PlayerState,
    amount: u64,
) -> Result<()> {
    require!(
        player_state.chip_stack >= amount,
        PokerError::InsufficientChips
    );
    Ok(())
}

/// Validate call amount
pub fn validate_call(
    game: &Game,
    player_state: &PlayerState,
) -> Result<u64> {
    let call_amount = game.current_bet.saturating_sub(player_state.current_bet);
    
    // If player doesn't have enough, they go all-in
    let actual_call = call_amount.min(player_state.chip_stack);
    
    Ok(actual_call)
}

/// Validate raise amount
pub fn validate_raise(
    game: &Game,
    player_state: &PlayerState,
    raise_amount: u64,
) -> Result<()> {
    // Total amount player needs to put in
    let call_amount = game.current_bet.saturating_sub(player_state.current_bet);
    let total_bet = call_amount + raise_amount;
    
    // Check sufficient chips
    validate_sufficient_chips(player_state, total_bet)?;
    
    // Minimum raise is 2x the current bet (or big blind if no bet yet)
    let min_raise = if game.current_bet == 0 {
        game.big_blind
    } else {
        game.current_bet * MIN_RAISE_MULTIPLIER
    };
    
    require!(
        raise_amount >= min_raise || total_bet == player_state.chip_stack,
        PokerError::InvalidBetAmount
    );
    
    Ok(())
}

/// Validate bet amount (for opening bet in a round)
pub fn validate_bet(
    game: &Game,
    player_state: &PlayerState,
    bet_amount: u64,
) -> Result<()> {
    // Must be at least big blind
    require!(
        bet_amount >= game.big_blind || bet_amount == player_state.chip_stack,
        PokerError::InvalidBetAmount
    );
    
    // Check sufficient chips
    validate_sufficient_chips(player_state, bet_amount)?;
    
    Ok(())
}

/// Validate check action (only valid if no bet to call)
pub fn validate_check(
    game: &Game,
    player_state: &PlayerState,
) -> Result<()> {
    require!(
        game.current_bet == player_state.current_bet,
        PokerError::InvalidAction
    );
    Ok(())
}

/// Validate fold action (always valid)
pub fn validate_fold() -> Result<()> {
    Ok(())
}

/// Validate all-in action
pub fn validate_all_in(
    player_state: &PlayerState,
) -> Result<u64> {
    require!(
        player_state.chip_stack > 0,
        PokerError::InsufficientChips
    );
    Ok(player_state.chip_stack)
}

/// Validate game is in correct stage for betting
pub fn validate_betting_stage(game: &Game) -> Result<()> {
    require!(
        matches!(
            game.stage,
            GameStage::PreFlop | GameStage::Flop | GameStage::Turn | GameStage::River
        ),
        PokerError::InvalidGameStage
    );
    Ok(())
}

/// Check if betting round is complete
pub fn is_betting_round_complete(
    game: &Game,
    player_states: &[PlayerState],
) -> bool {
    let mut active_count = 0;
    let mut acted_count = 0;
    let mut max_bet = 0u64;
    
    for i in 0..game.player_count as usize {
        if !game.active_players[i] {
            continue;
        }
        
        let player_state = &player_states[i];
        
        // Skip folded and all-in players
        if player_state.has_folded || player_state.is_all_in {
            continue;
        }
        
        active_count += 1;
        
        if player_state.current_bet > max_bet {
            max_bet = player_state.current_bet;
        }
        
        // Player has acted and matched the current bet
        if player_state.current_bet == game.current_bet {
            acted_count += 1;
        }
    }
    
    // Round complete if all active players have acted and matched the bet
    active_count > 0 && acted_count == active_count
}

/// Validate player action timeout
pub fn validate_action_timeout(
    game: &Game,
    current_time: i64,
) -> Result<()> {
    let time_since_last_action = current_time - game.last_action_at;
    
    require!(
        time_since_last_action < TURN_TIMEOUT,
        PokerError::InvalidAction
    );
    
    Ok(())
}