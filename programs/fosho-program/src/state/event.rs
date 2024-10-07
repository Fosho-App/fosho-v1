use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Event {
  pub community: Pubkey,
  pub reward_mint: Option<Pubkey>,
  pub commitment_fee: u64,
  pub event_start_time: i64,
  pub max_attendees: u32,
  pub registration_end_time: i64,
  pub current_attendees: u32,
  pub bump: u8,
  pub nonce: u32,
  pub reward_per_user: u64,
  pub is_cancelled: bool
}