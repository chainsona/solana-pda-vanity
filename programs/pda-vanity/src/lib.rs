use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

declare_id!("7d4pygUVej17wWKY6uiPdFSVPTDKEEAzR4YMmkc1Bss1");

#[program]
pub mod pda_vanity {
    use super::*;

    #[allow(unused_variables)]
    pub fn create_vanity_token(
        ctx: Context<CreateVanityToken>,
        vanity_seed: u64,
        decimals: u8,
    ) -> Result<()> {
        let mint_key = ctx.accounts.mint.key();
        
        // Enforce the vanity suffix on-chain
        // Note: to_string() performs Base58 encoding which costs Compute Units,
        // but it is necessary to verify the string representation.
        let mint_string = mint_key.to_string();
        require!(
            mint_string.ends_with("pump"),
            ErrorCode::InvalidVanityAddress
        );

        msg!("Created vanity token mint: {}", mint_key);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(vanity_seed: u64, decimals: u8)]
pub struct CreateVanityToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [vanity_seed.to_le_bytes().as_ref()],
        bump,
        mint::decimals = decimals,
        mint::authority = payer,
    )]
    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The derived PDA does not end with the required suffix 'pump'.")]
    InvalidVanityAddress,
}
