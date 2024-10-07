use anchor_lang::prelude::*;
use crate::{constant::*, state::*};

#[derive(Accounts)]
pub struct VerifyAttendee<'info> {
  #[account(
    mut,
    has_one = event
  )]
  pub attendee_record: Account<'info, Attendee>,
  #[account(
    seeds = [
      EVENT_PRE_SEED.as_ref(),
      community.key().as_ref(),
      &event.nonce.to_le_bytes()
    ],
    bump = event.bump,
    has_one = community,
  )]
  pub event: Account<'info, Event>,
  #[account(
    seeds = [
      COMMUNITY_PRE_SEED.as_ref(),
      community.seed.as_ref(),
    ],
    bump = community.bump,
    has_one = authority
  )]
  pub community: Account<'info, Community>,
  pub authority: Signer<'info>
}

pub fn verify_attendee_handler(
  ctx: Context<VerifyAttendee>
) -> Result<()> {
  let attendee_record = &mut ctx.accounts.attendee_record;
  attendee_record.status = AttendeeStatus::Verified;
  Ok(())
}