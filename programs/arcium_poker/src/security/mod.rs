// Security and anti-cheat module - Module 7
pub mod validation;
pub mod integrity;
pub mod zkp;

// Export specific items
pub use validation::{
    validate_game_state,
    validate_chip_conservation,
    validate_deck_integrity,
    validate_state_transition,
};
pub use integrity::{
    check_collusion_prevention,
    verify_shuffle_randomness,
    audit_game_actions,
};
pub use zkp::{
    verify_hand_proof,
    generate_shuffle_proof,
    verify_shuffle_proof,
};
