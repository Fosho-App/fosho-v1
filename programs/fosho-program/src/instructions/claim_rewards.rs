use crate::{constant::*, error::FoshoErrors, state::*, utils::get_event_ends_at_from_attributes};
use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use mpl_core::{
  accounts::BaseCollectionV1,
  fetch_plugin,
  types::{Attributes, PluginType},
};

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
  #[account(
      mut,
      constraint = event_collection.update_authority == community.key(),
  )]
  pub event_collection: Box<Account<'info, BaseCollectionV1>>,
  #[account(mut)]
  pub claimer: Signer<'info>,
  pub token_program: Interface<'info, TokenInterface>,
  pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimRewards<'info> {
  pub fn claim_reward_tokens(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
    let cpi_accounts = TransferChecked {
      from: self.reward_account.as_ref().unwrap().to_account_info(),
      to: self.receiver_account.as_ref().unwrap().to_account_info(),
      mint: self.reward_mint.as_ref().unwrap().to_account_info(),
      authority: self.event.to_account_info(),
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

pub fn claim_rewards_handler(ctx: Context<ClaimRewards>) -> Result<()> {
  let attendee_record = &mut ctx.accounts.attendee_record;
  let claimer = ctx.accounts.claimer.key();
  let community = &ctx.accounts.community;
  let event = &ctx.accounts.event;

  if event.is_cancelled {
    attendee_record.status = AttendeeStatus::Verified;
  }

  match attendee_record.status {
    AttendeeStatus::Pending => {
      // community_authority has to be able to claim the reward/commitement_fee if the attendee did not attend the event
      // if and only if the event ended.
      if claimer != community.authority {
        return Err(FoshoErrors::AttendeeStatusPending.into());
      }
      let (_, collection_attribute_list, _) = fetch_plugin::<BaseCollectionV1, Attributes>(
        &ctx.accounts.event_collection.to_account_info(),
        PluginType::Attributes,
      )?;
      let event_ends_at =
        get_event_ends_at_from_attributes(&collection_attribute_list.attribute_list)?;

      let current_unix_ts = Clock::get()?.unix_timestamp as u64;
      if event_ends_at.ne(&0) {
        require!(
          current_unix_ts <= event_ends_at,
          FoshoErrors::EventHasNotEnded
        );
      }
    }
    AttendeeStatus::Claimed => {
      return Err(FoshoErrors::AlreadyClaimed.into());
    }
    AttendeeStatus::Rejected => {
      require_keys_eq!(claimer, community.authority, FoshoErrors::InvalidClaimer);
    }
    AttendeeStatus::Verified => {
      require_keys_eq!(claimer, attendee_record.owner, FoshoErrors::InvalidClaimer);
    }
  }

  attendee_record.status = AttendeeStatus::Claimed;

  if event.reward_per_user.gt(&0) {
    let reward_mint = ctx.accounts.reward_mint.as_ref();

    if ctx.accounts.reward_mint.is_none()
      || ctx.accounts.reward_account.is_none()
      || ctx.accounts.receiver_account.is_none()
    {
      return Err(FoshoErrors::AccountNotProvided.into());
    }

    transfer_checked(
      ctx.accounts.claim_reward_tokens(),
      event.reward_per_user,
      reward_mint.unwrap().decimals,
    )?;
  }
  
  if event.commitment_fee.gt(&0) {
    ctx.accounts.claim_commitment_fee(event.commitment_fee)?;
  }

  Ok(())
}
