use anchor_lang::prelude::*;
use crate::{constant::*, state::*};

#[derive(Accounts)]
pub struct CancelEvent<'info> {
  #[account(
    mut,
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

pub fn cancel_event_handler(
  ctx: Context<CancelEvent>
) -> Result<()> {
  let event = &mut ctx.accounts.event;
  event.is_cancelled = true;
  Ok(())
}