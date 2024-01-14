use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

use crate::states::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64, initializer_amount: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    #[account(
        mut,
        constraint = initializer_ata_a.amount >= initializer_amount,
        associated_token::mint = mint_a,
        associated_token::authority = initializer
    )]
    pub initializer_ata_a: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = initializer,
        space = Escrow::INIT_SPACE,
        seeds = [b"state".as_ref(), &seed.to_le_bytes()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize_escrow(
        &mut self,
        seed: u64,
        bumps: &InitializeBumps,
        initializer_amount: u64,
        taker_amount: u64,
    ) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,
            bump: bumps.escrow,
            initializer: self.initializer.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            initializer_amount,
            taker_amount,
        });
        Ok(())
    }

    pub fn deposit(&mut self, initializer_amount: u64) -> Result<()> {
        transfer_checked(
            self.into_deposit_context(),
            initializer_amount,
            self.mint_a.decimals,
        )
    }

    fn into_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.initializer_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.initializer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
