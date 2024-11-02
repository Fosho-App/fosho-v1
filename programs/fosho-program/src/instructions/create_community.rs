use crate::{constant::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(
  seed: Pubkey
)]
pub struct CreateCommunity<'info> {
  #[account(
    init,
    seeds = [
      COMMUNITY_PRE_SEED.as_ref(),
      seed.as_ref()
    ],
    bump,
    payer = payer,
    space = 8 + Community::INIT_SPACE
  )]
  pub community: Account<'info, Community>,
  pub authority: Signer<'info>,
  #[account(mut)]
  pub payer: Signer<'info>,
  pub system_program: Program<'info, System>,
}

pub fn create_community_handler(
  ctx: Context<CreateCommunity>,
  seed: Pubkey,
  name: String,
) -> Result<()> {
  let community = &mut ctx.accounts.community;
  community.authority = ctx.accounts.authority.key();
  community.events_count = 0;
  community.bump = ctx.bumps.community;
  community.seed = seed;
  community.name = name;
  Ok(())
}
