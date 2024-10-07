use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};
use crate::{constant::*, error::FoshoErrors, state::*};

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
  #[account(
    mut,
    has_one = event
  )]
  pub attendee_record: Account<'info, Attendee>,
  #[account(
    mut,
    seeds = [
      EVENT_PRE_SEED.as_ref(),
      community.key().as_ref(),
      &event.nonce.to_le_bytes()
    ],
    bump = event.bump,
  )]
  pub event: Account<'info, Event>,
  #[account(
    seeds = [
      COMMUNITY_PRE_SEED.as_ref(),
      community.seed.as_ref(),
    ],
    bump = community.bump
  )]
  pub community: Account<'info, Community>,
  #[account(
    mint::token_program = token_program,
  )]
  pub reward_mint: Option<InterfaceAccount<'info, Mint>>,
  #[account(
    mut,
    associated_token::mint = reward_mint,
    associated_token::authority = event,
    associated_token::token_program = token_program
  )]
  pub reward_account: Option<InterfaceAccount<'info, TokenAccount>>,
  #[account(
    mut,
    associated_token::mint = reward_mint,
    associated_token::authority = claimer,
    associated_token::token_program = token_program
  )]
  pub receiver_account: Option<InterfaceAccount<'info, TokenAccount>>,
  #[account(mut)]
  pub claimer: Signer<'info>,
  pub token_program: Interface<'info, TokenInterface>,
  pub associated_token_program: Program<'info, AssociatedToken>
}

impl<'info> ClaimRewards<'info> {
  pub fn claim_reward_tokens(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
    let cpi_accounts = TransferChecked {
      from: self.reward_account.as_ref().unwrap().to_account_info(),
      to: self.receiver_account.as_ref().unwrap().to_account_info(),
      mint: self.reward_mint.as_ref().unwrap().to_account_info(),
      authority: self.event.to_account_info()
    };

    let cpi_program = self.token_program.to_account_info();

    CpiContext::new(cpi_program, cpi_accounts)
  }
  
  pub fn claim_commitment_fee(&self, commitment_fee: u64) -> Result<()> {
    self.event.sub_lamports(commitment_fee)?;
    self.claimer.add_lamports(commitment_fee)?;
    Ok(())
  }
}

pub fn claim_rewards_handler(
  ctx: Context<ClaimRewards>
) -> Result<()> {
  let attendee_record = &ctx.accounts.attendee_record;
  let claimer = ctx.accounts.claimer.key();
  let community = &ctx.accounts.community;
  let event = &ctx.accounts.event;

  match attendee_record.status {
    AttendeeStatus::Pending => { 
      return Err(FoshoErrors::AttendeeStatusPending.into());
    },
    AttendeeStatus::Rejected => {
      require_keys_eq!(claimer, community.authority, FoshoErrors::InvalidClaimer);
    },
    AttendeeStatus::Verified => {
      require_keys_eq!(claimer, attendee_record.owner, FoshoErrors::InvalidClaimer);
    }
  }

  if event.reward_per_user.gt(&0) {
    let reward_mint = ctx.accounts.reward_mint.as_ref();

    if 
      ctx.accounts.reward_mint.is_none() || 
      ctx.accounts.reward_account.is_none() ||
      ctx.accounts.receiver_account.is_none()
    {
      return Err(FoshoErrors::AccountNotProvided.into());
    }

    transfer_checked(
      ctx.accounts.claim_reward_tokens(), 
      event.reward_per_user, 
      reward_mint.unwrap().decimals
    )?;

  }

  ctx.accounts.claim_commitment_fee(event.commitment_fee)?;
  Ok(())
}