use anchor_lang::prelude::*;

#[error_code]
pub enum PokerError {
    #[msg("Game is full, cannot join")]
    GameFull,
    
    #[msg("Not enough players to start game")]
    NotEnoughPlayers,
    
    #[msg("Game has already started")]
    GameAlreadyStarted,
    
    #[msg("Game is not in the correct stage for this action")]
    InvalidGameStage,
    
    #[msg("Player is not in this game")]
    PlayerNotInGame,
    
    #[msg("Player already in game")]
    PlayerAlreadyInGame,
    
    #[msg("Insufficient balance for buy-in")]
    InsufficientBalance,
    
    #[msg("Buy-in amount too low")]
    BuyInTooLow,
    
    #[msg("Buy-in amount too high")]
    BuyInTooHigh,
    
    #[msg("Invalid bet amount")]
    InvalidBetAmount,
    
    #[msg("Not player's turn")]
    NotPlayerTurn,
    
    #[msg("Invalid action for current game state")]
    InvalidAction,
    
    #[msg("Player has insufficient chips")]
    InsufficientChips,
    
    #[msg("Invalid seat position")]
    InvalidSeatPosition,
    
    #[msg("Seat is already occupied")]
    SeatOccupied,
    
    #[msg("Cannot leave during active hand")]
    CannotLeaveDuringHand,
    
    #[msg("Deck not initialized")]
    DeckNotInitialized,
    
    #[msg("Cards not dealt yet")]
    CardsNotDealt,
    
    #[msg("Invalid card index")]
    InvalidCardIndex,
    
    #[msg("Arcium MPC operation failed")]
    ArciumMpcFailed,
    
    #[msg("Encryption/Decryption failed")]
    EncryptionFailed,
    
    #[msg("Invalid game configuration")]
    InvalidGameConfig,
    
    #[msg("Game has not finished")]
    GameNotFinished,
}