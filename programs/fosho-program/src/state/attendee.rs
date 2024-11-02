use anchor_lang::prelude::*;

#[account]
pub struct Attendee {
  pub event: Pubkey,
  pub owner: Pubkey,
  pub bump: u8,
  pub status: AttendeeStatus
}

impl Attendee {
  pub const ATTENDEE_SIZE: usize = 32 + 32 + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum AttendeeStatus {
  Pending,
  Verified,
  Rejected,
  Claimed
}