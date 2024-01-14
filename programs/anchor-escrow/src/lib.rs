use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, TransferChecked};

declare_id!("D3B39yETxmG29V3QPPNsdEzxQyonU5tjiwEr3svnMoRt");

mod contexts;
use contexts::*;
mod states;

#[program]
pub mod anchor_escrow {
    use super::*;

    const AUTHORITY_SEED: &[u8] = b"authority"; // To be deprecated

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        initializer_amount: u64,
        taker_amount: u64,
    ) -> Result<()> {
        ctx.accounts
            .initialize_escrow(seed, &ctx.bumps, initializer_amount, taker_amount)?;
        ctx.accounts.deposit(initializer_amount)?;

        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()?;

        // let authority_seeds = &[
        //     &AUTHORITY_SEED[..],
        //     &[ctx.accounts.escrow_state.vault_authority_bump],
        // ];

        // token::transfer_checked(
        //     ctx.accounts
        //         .into_transfer_to_initializer_context()
        //         .with_signer(&[&authority_seeds[..]]),
        //     ctx.accounts.escrow_state.initializer_amount,
        //     ctx.accounts.mint.decimals,
        // )?;

        // token::close_account(
        //     ctx.accounts
        //         .into_close_context()
        //         .with_signer(&[&authority_seeds[..]]),
        // )?;

        Ok(())
    }

    pub fn exchange(ctx: Context<Exchange>) -> Result<()> {
        let authority_seeds = &[
            &AUTHORITY_SEED[..],
            &[ctx.accounts.escrow_state.vault_authority_bump],
        ];

        token::transfer_checked(
            ctx.accounts.into_transfer_to_initializer_context(),
            ctx.accounts.escrow_state.taker_amount,
            ctx.accounts.taker_deposit_token_mint.decimals,
        )?;

        token::transfer_checked(
            ctx.accounts
                .into_transfer_to_taker_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_state.initializer_amount,
            ctx.accounts.initializer_deposit_token_mint.decimals,
        )?;

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        Ok(())
    }
}

// #[derive(Accounts)]
// #[instruction(escrow_seed: u64, initializer_amount: u64)]
// pub struct Initialize<'info> {
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     #[account(mut)]
//     pub initializer: Signer<'info>,
//     pub mint: Account<'info, Mint>,
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     #[account(
//         seeds = [b"authority".as_ref()],
//         bump,
//     )]
//     pub vault_authority: AccountInfo<'info>,
//     #[account(
//         init,
//         payer = initializer,
//         associated_token::mint = mint,
//         associated_token::authority = vault_authority
//     )]
//     pub vault: Box<Account<'info, TokenAccount>>,
//     #[account(
//         mut,
//         constraint = initializer_deposit_token_account.amount >= initializer_amount
//     )]
//     pub initializer_deposit_token_account: Account<'info, TokenAccount>,
//     pub initializer_receive_token_account: Account<'info, TokenAccount>,
//     #[account(
//         init,
//         seeds = [b"state".as_ref(), &escrow_seed.to_le_bytes()],
//         bump,
//         payer = initializer,
//         space = EscrowState::space()
//     )]
//     pub escrow_state: Box<Account<'info, EscrowState>>,
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     pub system_program: Program<'info, System>,
//     pub rent: Sysvar<'info, Rent>,
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     pub token_program: Program<'info, Token>,
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     pub associated_token_program: Program<'info, AssociatedToken>,
// }

// #[derive(Accounts)]
// pub struct Cancel<'info> {
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     #[account(mut)]
//     pub initializer: Signer<'info>,
//     pub mint: Account<'info, Mint>,
//     #[account(mut)]
//     pub vault: Account<'info, TokenAccount>,
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     #[account(
//         seeds = [b"authority".as_ref()],
//         bump,
//     )]
//     pub vault_authority: AccountInfo<'info>,
//     #[account(mut)]
//     pub initializer_deposit_token_account: Account<'info, TokenAccount>,
//     #[account(
//         mut,
//         constraint = escrow_state.initializer_key == *initializer.key,
//         constraint = escrow_state.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
//         close = initializer
//     )]
//     pub escrow_state: Box<Account<'info, EscrowState>>,
//     /// CHECK: This is not dangerous because we don't read or write from this account
//     pub token_program: Program<'info, Token>,
// }

#[derive(Accounts)]
pub struct Exchange<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub taker: Signer<'info>,
    pub initializer_deposit_token_mint: Account<'info, Mint>,
    pub taker_deposit_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub taker_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub taker_receive_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_receive_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub initializer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_state.taker_amount <= taker_deposit_token_account.amount,
        constraint = escrow_state.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        constraint = escrow_state.initializer_receive_token_account == *initializer_receive_token_account.to_account_info().key,
        constraint = escrow_state.initializer_key == *initializer.key,
        close = initializer
    )]
    pub escrow_state: Box<Account<'info, EscrowState>>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        seeds = [b"authority".as_ref()],
        bump,
    )]
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct EscrowState {
    pub random_seed: u64,
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub initializer_receive_token_account: Pubkey,
    pub initializer_amount: u64,
    pub taker_amount: u64,
    pub vault_authority_bump: u8,
}

// impl EscrowState {
//     pub fn space() -> usize {
//         8 + 121
//     }
// }

// impl<'info> Initialize<'info> {
//     fn into_transfer_to_pda_context(
//         &self,
//     ) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
//         let cpi_accounts = TransferChecked {
//             from: self.initializer_deposit_token_account.to_account_info(),
//             mint: self.mint.to_account_info(),
//             to: self.vault.to_account_info(),
//             authority: self.initializer.to_account_info(),
//         };
//         CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
//     }
// }

// impl<'info> Cancel<'info> {
//     fn into_transfer_to_initializer_context(
//         &self,
//     ) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
//         let cpi_accounts = TransferChecked {
//             from: self.vault.to_account_info(),
//             mint: self.mint.to_account_info(),
//             to: self.initializer_deposit_token_account.to_account_info(),
//             authority: self.vault_authority.clone(),
//         };
//         CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
//     }

//     fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
//         let cpi_accounts = CloseAccount {
//             account: self.vault.to_account_info(),
//             destination: self.initializer.to_account_info(),
//             authority: self.vault_authority.clone(),
//         };
//         CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
//     }
// }

impl<'info> Exchange<'info> {
    fn into_transfer_to_initializer_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.taker_deposit_token_account.to_account_info(),
            mint: self.taker_deposit_token_mint.to_account_info(),
            to: self.initializer_receive_token_account.to_account_info(),
            authority: self.taker.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_taker_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.initializer_deposit_token_mint.to_account_info(),
            to: self.taker_receive_token_account.to_account_info(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.initializer.clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
