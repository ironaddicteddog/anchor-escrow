use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        close_account, transfer_checked, CloseAccount, Mint, Token, TokenAccount, TransferChecked,
    },
};

use crate::states::Escrow;

#[derive(Accounts)]
pub struct Cancel<'info> {
    #[account(mut)]
    initializer: Signer<'info>,
    mint_a: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = initializer
    )]
    initializer_ata_a: Account<'info, TokenAccount>,
    #[account(
        mut,
        has_one = initializer,
        has_one = mint_a,
        close = initializer,
        seeds=[b"state", escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    escrow: Account<'info, Escrow>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Account<'info, TokenAccount>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

impl<'info> Cancel<'info> {
    pub fn refund_and_close_vault(&mut self) -> Result<()> {
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"state",
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        transfer_checked(
            self.into_refund_context().with_signer(&signer_seeds),
            self.escrow.initializer_amount,
            self.mint_a.decimals,
        )?;

        close_account(self.into_close_context().with_signer(&signer_seeds))
    }

    fn into_refund_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.initializer_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.initializer.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
