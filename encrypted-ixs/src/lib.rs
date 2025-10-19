use arcis_imports::*;

#[encrypted]
mod circuits {
    use arcis_imports::*;

    // ============================================================================
    // POKER MPC CIRCUITS - Encrypted Texas Hold'em
    // ============================================================================
    // These circuits run in Multi-Party Computation (MPC) to ensure:
    // 1. Fair deck shuffling without any single party controlling the outcome
    // 2. Encrypted card dealing where only the recipient can decrypt
    // 3. Secure card reveal at showdown
    // ============================================================================

    /// Input for deck shuffling
    /// Each player contributes entropy to ensure fairness
    pub struct ShuffleInput {
        entropy_p1: [u8; 32],  // Player 1 entropy
        entropy_p2: [u8; 32],  // Player 2 entropy
        entropy_p3: [u8; 32],  // Player 3 entropy (can be zero if < 3 players)
        entropy_p4: [u8; 32],  // Player 4 entropy (can be zero if < 4 players)
        entropy_p5: [u8; 32],  // Player 5 entropy (can be zero if < 5 players)
        entropy_p6: [u8; 32],  // Player 6 entropy (can be zero if < 6 players)
    }

    /// Shuffle a 52-card deck using Fisher-Yates algorithm in MPC
    /// This ensures no single party can predict or manipulate the shuffle
    #[instruction]
    pub fn shuffle_deck(input_ctxt: Enc<Shared, ShuffleInput>) -> Enc<Shared, [u8; 52]> {
        let input = input_ctxt.to_arcis();
        
        // Combine all player entropy via addition (mod 256)
        let mut combined_entropy = input.entropy_p1;
        for i in 0..32 {
            combined_entropy[i] = (combined_entropy[i] as u16
                + input.entropy_p2[i] as u16
                + input.entropy_p3[i] as u16
                + input.entropy_p4[i] as u16
                + input.entropy_p5[i] as u16
                + input.entropy_p6[i] as u16) as u8;
        }
        
        // Initialize ordered deck (0-51)
        let mut deck = [0u8; 52];
        for i in 0..52 {
            deck[i] = i as u8;
        }
        
        // Fisher-Yates shuffle using combined entropy
        for i in (1..52).rev() {
            // Generate pseudo-random index from entropy
            let entropy_idx = (i % 32) as usize;
            let random_byte = combined_entropy[entropy_idx];
            let j = (random_byte as usize) % (i + 1);
            
            // Swap deck[i] with deck[j]
            let temp = deck[i];
            deck[i] = deck[j];
            deck[j] = temp;
            
            // Mix entropy for next iteration (simple hash)
            combined_entropy = hash_entropy(combined_entropy);
        }
        
        // Return shuffled deck (shared among all MPC nodes)
        input_ctxt.owner.from_arcis(deck)
    }

    /// Input for dealing a card to a specific player
    pub struct DealCardInput {
        shuffled_deck: [u8; 52],  // The shuffled deck
        card_index: u8,            // Which card to deal (0-51)
    }

    /// Deal a card from the shuffled deck
    /// The card is encrypted and only the recipient can decrypt it
    #[instruction]
    pub fn deal_card(
        input_ctxt: Enc<Shared, DealCardInput>
    ) -> Enc<Shared, u8> {
        let input = input_ctxt.to_arcis();
        
        // Get the card at the specified index
        let card = input.shuffled_deck[input.card_index as usize];
        
        // Return encrypted card
        input_ctxt.owner.from_arcis(card)
    }

    /// Input for revealing multiple cards at showdown
    pub struct RevealCardsInput {
        card1: u8,  // First hole card
        card2: u8,  // Second hole card
    }

    /// Reveal hole cards at showdown
    /// This decrypts the cards so everyone can see them
    #[instruction]
    pub fn reveal_hole_cards(
        input_ctxt: Enc<Shared, RevealCardsInput>
    ) -> Enc<Shared, [u8; 2]> {
        let input = input_ctxt.to_arcis();
        
        let cards = [input.card1, input.card2];
        
        // Return revealed cards
        input_ctxt.owner.from_arcis(cards)
    }

    /// Generate random number for tie-breaking
    pub struct RandomInput {
        seed: [u8; 32],
        max_value: u8,
    }

    #[instruction]
    pub fn generate_random(
        input_ctxt: Enc<Shared, RandomInput>
    ) -> Enc<Shared, u8> {
        let input = input_ctxt.to_arcis();
        
        // Simple random generation from seed
        let mut hash = input.seed;
        hash = hash_entropy(hash);
        
        let random_value = hash[0] % input.max_value;
        
        input_ctxt.owner.from_arcis(random_value)
    }

    // ============================================================================
    // HELPER FUNCTIONS
    // ============================================================================

    /// Simple entropy mixing function
    /// In production, this would use a proper cryptographic hash
    fn hash_entropy(input: [u8; 32]) -> [u8; 32] {
        let mut output = input;
        
        // Simple mixing (XOR with rotated values)
        for i in 0..32 {
            let prev_idx = if i == 0 { 31 } else { i - 1 };
            let next_idx = if i == 31 { 0 } else { i + 1 };
            
            let prev = input[prev_idx];
            let curr = input[i];
            let next = input[next_idx];
            
            // Simple arithmetic mixing (avoiding wrapping_* methods)
            output[i] = (prev * 7 + curr * 13 + next * 17) as u8;
        }
        
        output
    }
}

