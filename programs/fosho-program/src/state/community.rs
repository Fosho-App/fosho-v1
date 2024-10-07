use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Community {
  pub seed: Pubkey,
  pub authority: Pubkey,
  pub events_count: u32,
  pub bump: u8
}