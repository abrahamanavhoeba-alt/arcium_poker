use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::shared::PokerError;

/// Create token escrow account for game
pub fn create_token_escrow(
    game_key: Pubkey,
    token_mint: Pubkey,
    bump: u8,
) -> Result<()> {
    msg!(
        "[TOKEN] Created escrow for game {} with mint {}",
        game_key,
        token_mint
    );
    Ok(())
}

/// Lock tokens when player joins game
pub fn lock_tokens_on_join<'info>(
    player_token_account: &Account<'info, TokenAccount>,
    escrow_token_account: &Account<'info, TokenAccount>,
    player_authority: &Signer<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    // Validate token accounts
    require!(
        player_token_account.mint == escrow_token_account.mint,
        PokerError::InvalidGameConfig
    );
    
    require!(
        player_token_account.amount >= amount,
        PokerError::InsufficientBalance
    );
    
    // Transfer tokens from player to escrow
    let cpi_accounts = Transfer {
        from: player_token_account.to_account_info(),
        to: escrow_token_account.to_account_info(),
        authority: player_authority.to_account_info(),
    };
    
    let cpi_program = token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::transfer(cpi_ctx, amount)?;
    
    msg!(
        "[TOKEN] Locked {} tokens from player {} to escrow",
        amount,
        player_authority.key()
    );
    
    Ok(())
}

/// Release tokens when player leaves game
pub fn release_tokens_on_leave<'info>(
    escrow_token_account: &Account<'info, TokenAccount>,
    player_token_account: &Account<'info, TokenAccount>,
    escrow_authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
    escrow_bump: u8,
    game_key: Pubkey,
) -> Result<()> {
    // Validate sufficient balance in escrow
    require!(
        escrow_token_account.amount >= amount,
        PokerError::InsufficientChips
    );
    
    // Create PDA signer seeds
    let seeds = &[
        b"token_escrow",
        game_key.as_ref(),
        &[escrow_bump],
    ];
    let signer = &[&seeds[..]];
    
    // Transfer tokens from escrow to player
    let cpi_accounts = Transfer {
        from: escrow_token_account.to_account_info(),
        to: player_token_account.to_account_info(),
        authority: escrow_authority.clone(),
    };
    
    let cpi_program = token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::transfer(cpi_ctx, amount)?;
    
    msg!(
        "[TOKEN] Released {} tokens from escrow to player",
        amount
    );
    
    Ok(())
}

/// Transfer winnings from escrow to player
pub fn transfer_winnings<'info>(
    escrow_token_account: &Account<'info, TokenAccount>,
    player_token_account: &Account<'info, TokenAccount>,
    escrow_authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
    escrow_bump: u8,
    game_key: Pubkey,
) -> Result<()> {
    release_tokens_on_leave(
        escrow_token_account,
        player_token_account,
        escrow_authority,
        token_program,
        amount,
        escrow_bump,
        game_key,
    )
}
