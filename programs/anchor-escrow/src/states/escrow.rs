use anchor_lang::prelude::*;

#[account]
pub struct Escrow {
    pub seed: u64,
    pub bump: u8,
    pub initializer: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub initializer_amount: u64,
    pub taker_amount: u64,
}

impl Space for Escrow {
    // First 8 Bytes are Discriminator (u64)
    const INIT_SPACE: usize = 8 + 8 + 1 + 32 + 32 + 32 + 8 + 8;
}

// impl EscrowState {
//     pub fn space() -> usize {
//         8 + 121
//     }
// }

// #[account]
// pub struct Escrow {
//     pub seed: u64,
//     pub mint_a: Pubkey,
//     pub mint_b: Pubkey,
//     pub receive: u64,
//     pub bump: u8,
// }
