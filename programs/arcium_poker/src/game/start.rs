use anchor_lang::prelude::*;
use super::state::Game;
use crate::player::state::PlayerState;
use crate::arcium::mpc_shuffle::{mpc_shuffle_deck, ShuffleParams};
use crate::arcium::mpc_deal::{mpc_deal_card, DealParams};
use crate::types::GameStage;
use crate::shared::{constants::*, PokerError};

/// Start the poker game - triggers MPC shuffle and deals hole cards
pub fn handler(
    ctx: Context<crate::StartGame>,
    player_entropy: Vec<[u8; 32]>, // Each player provides randomness
) -> Result<()> {
    let game = &mut ctx.accounts.game;
    
    // Validate game can start
    require!(
        game.stage == GameStage::Waiting,
        PokerError::GameAlreadyStarted
    );
    require!(
        game.player_count >= MIN_PLAYERS as u8,
        PokerError::NotEnoughPlayers
    );
    require!(
        player_entropy.len() == game.player_count as usize,
        PokerError::InvalidGameConfig
    );
    
    msg!("[GAME START] Starting game {} with {} players", game.game_id, game.player_count);
    
    // ========================================================================
    // STEP 1: ARCIUM MPC SHUFFLE 🔐
    // ========================================================================
    msg!("[ARCIUM MPC] Initiating secure shuffle...");
    
    // Collect all player pubkeys
    let players: Vec<Pubkey> = game.players[..game.player_count as usize]
        .iter()
        .copied()
        .collect();
    
    // Perform MPC shuffle with all players contributing entropy
    let shuffle_params = ShuffleParams {
        player_pubkeys: players.clone(),
        player_entropy: player_entropy.clone(),
        game_id: game.game_id,
    };
    
    let shuffle_result = mpc_shuffle_deck(shuffle_params)?;
    
    msg!(
        "[ARCIUM MPC] Shuffle complete! Session ID: {:?}",
        &shuffle_result.session_id[..8]
    );
    msg!(
        "[ARCIUM MPC] Commitment: {:?}",
        &shuffle_result.commitment[..8]
    );
    
    // Store shuffle result in game state
    game.encrypted_deck = shuffle_result.session_id;
    game.deck_initialized = true;
    
    // ========================================================================
    // STEP 2: DEAL ENCRYPTED HOLE CARDS 🎴
    // ========================================================================
    msg!("[DEALING] Dealing encrypted hole cards to all players...");
    
    // Deal 2 hole cards to each player (encrypted via Arcium MPC)
    let mut card_index = 0u8;
    
    for (i, player_account) in ctx.remaining_accounts.iter().enumerate() {
        if i >= game.player_count as usize {
            break;
        }
        
        let player_pubkey = game.players[i];
        msg!("[DEALING] Dealing to player {} at seat {}", player_pubkey, i);
        
        // Deal hole cards using Arcium MPC
        for hole_card_num in 0..HOLE_CARDS {
            let deal_params = DealParams {
                card_index: shuffle_result.shuffled_indices[card_index as usize],
                player: player_pubkey,
                session_id: shuffle_result.session_id,
                game_id: game.game_id,
            };
            
            let encrypted_card = mpc_deal_card(deal_params)?;
            
            msg!(
                "[DEALING] Card {}/{} dealt to seat {} (encrypted index: {})",
                hole_card_num + 1,
                HOLE_CARDS,
                i,
                encrypted_card.encrypted_index
            );
            
            card_index += 1;
        }
    }
    
    // ========================================================================
    // STEP 3: INITIALIZE GAME STATE
    // ========================================================================
    
    // Set game stage to PreFlop
    game.stage = GameStage::PreFlop;
    
    // Set dealer button (starts at position 0)
    game.dealer_position = 0;
    
    // First player after big blind acts first
    // Small blind = dealer + 1, Big blind = dealer + 2, First actor = dealer + 3
    game.current_player_index = (game.dealer_position + 3) % game.player_count;
    
    // Set timestamp
    game.started_at = Clock::get()?.unix_timestamp;
    game.last_action_at = game.started_at;
    
    // Reset pot and bets
    game.pot = 0;
    game.current_bet = game.big_blind;
    
    msg!("[GAME START] Game initialized!");
    msg!("[GAME START] Dealer button at seat {}", game.dealer_position);
    msg!("[GAME START] Current player: seat {}", game.current_player_index);
    msg!("[GAME START] Stage: {:?}", game.stage);
    
    // TODO: Post blinds automatically in next PR
    msg!("[GAME START] Note: Blinds must be posted by players");
    
    Ok(())
}

// StartGame struct moved to lib.rs at crate root (required by Anchor)