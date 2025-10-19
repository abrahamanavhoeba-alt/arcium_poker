use anchor_lang::prelude::*;
use crate::types::{Suit, Rank};
use crate::shared::constants::DECK_SIZE;

/// Represents a playing card
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }
    
    /// Convert card to index (0-51)
    pub fn to_index(&self) -> u8 {
        let suit_offset = match self.suit {
            Suit::Hearts => 0,
            Suit::Diamonds => 13,
            Suit::Clubs => 26,
            Suit::Spades => 39,
        };
        let rank_offset = self.rank as u8 - 2; // Rank starts at 2
        suit_offset + rank_offset
    }
    
    /// Create card from index (0-51)
    pub fn from_index(index: u8) -> Result<Self> {
        require!(index < 52, crate::shared::PokerError::InvalidCardIndex);
        
        let suit = match index / 13 {
            0 => Suit::Hearts,
            1 => Suit::Diamonds,
            2 => Suit::Clubs,
            3 => Suit::Spades,
            _ => return Err(crate::shared::PokerError::InvalidCardIndex.into()),
        };
        
        let rank = match (index % 13) + 2 {
            2 => Rank::Two,
            3 => Rank::Three,
            4 => Rank::Four,
            5 => Rank::Five,
            6 => Rank::Six,
            7 => Rank::Seven,
            8 => Rank::Eight,
            9 => Rank::Nine,
            10 => Rank::Ten,
            11 => Rank::Jack,
            12 => Rank::Queen,
            13 => Rank::King,
            14 => Rank::Ace,
            _ => return Err(crate::shared::PokerError::InvalidCardIndex.into()),
        };
        
        Ok(Card { suit, rank })
    }
}

/// Encrypted deck state (stored in Game account)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EncryptedDeck {
    /// Encrypted card indices (shuffled order)
    /// Each element is encrypted card index pointing to original deck position
    pub encrypted_indices: [u8; DECK_SIZE],
    
    /// MPC commitment/hash of the shuffled deck
    /// Used for verification that shuffle was done correctly
    pub shuffle_commitment: [u8; 32],
    
    /// Next card index to deal from deck
    pub next_card_index: u8,
    
    /// Number of cards dealt
    pub cards_dealt: u8,
    
    /// Shuffle round/session ID from Arcium
    pub shuffle_session_id: [u8; 32],
}

impl Default for EncryptedDeck {
    fn default() -> Self {
        Self {
            encrypted_indices: [0; DECK_SIZE],
            shuffle_commitment: [0; 32],
            next_card_index: 0,
            cards_dealt: 0,
            shuffle_session_id: [0; 32],
        }
    }
}

impl EncryptedDeck {
    /// Initialize deck with encrypted indices from Arcium MPC shuffle
    pub fn initialize_from_shuffle(
        encrypted_indices: [u8; DECK_SIZE],
        shuffle_commitment: [u8; 32],
        shuffle_session_id: [u8; 32],
    ) -> Self {
        Self {
            encrypted_indices,
            shuffle_commitment,
            next_card_index: 0,
            cards_dealt: 0,
            shuffle_session_id,
        }
    }
    
    /// Get next card index to deal (still encrypted)
    pub fn get_next_encrypted_card(&mut self) -> Result<u8> {
        require!(
            self.next_card_index < DECK_SIZE as u8,
            crate::shared::PokerError::InvalidCardIndex
        );
        
        let card_index = self.encrypted_indices[self.next_card_index as usize];
        self.next_card_index += 1;
        self.cards_dealt += 1;
        
        Ok(card_index)
    }
    
    /// Burn a card (deal it but don't reveal)
    pub fn burn_card(&mut self) -> Result<()> {
        self.get_next_encrypted_card()?;
        Ok(())
    }
    
    /// Check if deck has enough cards
    pub fn has_cards(&self, count: u8) -> bool {
        self.next_card_index + count <= DECK_SIZE as u8
    }
}

/// Generate standard 52-card deck (unshuffled)
pub fn generate_standard_deck() -> [Card; DECK_SIZE] {
    let mut deck = [Card { suit: Suit::Hearts, rank: Rank::Two }; DECK_SIZE];
    let mut index = 0;
    
    for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
        for rank_val in 2..=14 {
            let rank = match rank_val {
                2 => Rank::Two,
                3 => Rank::Three,
                4 => Rank::Four,
                5 => Rank::Five,
                6 => Rank::Six,
                7 => Rank::Seven,
                8 => Rank::Eight,
                9 => Rank::Nine,
                10 => Rank::Ten,
                11 => Rank::Jack,
                12 => Rank::Queen,
                13 => Rank::King,
                14 => Rank::Ace,
                _ => Rank::Two,
            };
            deck[index] = Card { suit, rank };
            index += 1;
        }
    }
    
    deck
}