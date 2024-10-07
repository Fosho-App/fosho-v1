use anchor_lang::prelude::*;
use crate::{constant::*, error::FoshoErrors, state::*};
use anchor_spl::{
  associated_token::AssociatedToken, 
  token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}
};

#[derive(Accounts)]
#[instruction(
  nonce: u32
)]
pub struct CreateEvent<'info> {
  #[account(
    init,
    seeds = [
      EVENT_PRE_SEED.as_ref(),
      community.key().as_ref(),
      &nonce.to_le_bytes()
    ],
    bump,
    payer = authority,
    space = 8 + Event::INIT_SPACE
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
  #[account(
    mint::token_program = token_program,
  )]
  pub reward_mint: Option<InterfaceAccount<'info, Mint>>,
  #[account(
    init,
    payer = authority,
    associated_token::mint = reward_mint,
    associated_token::authority = event,
    associated_token::token_program = token_program
  )]
  pub reward_account: Option<InterfaceAccount<'info, TokenAccount>>,
  #[account(
    mut,
    associated_token::mint = reward_mint,
    associated_token::authority = authority,
    associated_token::token_program = token_program
  )]
  pub sender_account: Option<InterfaceAccount<'info, TokenAccount>>,
  #[account(mut)]
  pub authority: Signer<'info>,
  pub token_program: Interface<'info, TokenInterface>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>
}

impl<'info> CreateEvent<'info> {
  pub fn deposit_reward_tokens(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
    
    let cpi_accounts = TransferChecked {
      from: self.sender_account.as_ref().unwrap().to_account_info(),
      to: self.reward_account.as_ref().unwrap().to_account_info(),
      mint: self.reward_mint.as_ref().unwrap().to_account_info(),
      authority: self.authority.to_account_info()
    };

    let cpi_program = self.token_program.to_account_info();

    CpiContext::new(cpi_program, cpi_accounts)
  }
}

pub fn create_event_handler(
  ctx: Context<CreateEvent>, 
  nonce: u32, 
  max_attendees: u32,
  commitment_fee: u64,
  event_start_time: i64,
  registration_end_time: Option<i64>,
  reward_per_user: u64
) -> Result<()> {
  let event = &mut ctx.accounts.event;

  let clock = Clock::get().unwrap();
  let current_time = clock.unix_timestamp;

  require_gt!(event_start_time, current_time, FoshoErrors::InvalidEventStartTime);

  event.registration_end_time = if let Some(reg_end_time) = registration_end_time {
    if reg_end_time.gt(&event_start_time) {
      return Err(FoshoErrors::InvalidRegistrationEndTime.into());
    }    
    reg_end_time
  } else {
    event_start_time
  };

  let reward_mint = ctx.accounts.reward_mint.as_ref();

  event.reward_mint = if let Some(reward_mint_acc) = reward_mint {
    Some(reward_mint_acc.key())
  } else {
    None
  };

  event.commitment_fee = commitment_fee;
  event.max_attendees = max_attendees;
  event.event_start_time = event_start_time;
  event.community = ctx.accounts.community.key();
  event.nonce = nonce;
  event.current_attendees = 0;
  event.bump = ctx.bumps.event;
  event.is_cancelled = false;
  
  if reward_per_user.gt(&0) {
    if 
      ctx.accounts.reward_mint.is_none() || 
      ctx.accounts.reward_account.is_none() ||
      ctx.accounts.sender_account.is_none()
    {
      return Err(FoshoErrors::AccountNotProvided.into());
    }

    let total_reward = reward_per_user.checked_mul(max_attendees as u64).unwrap();

    transfer_checked(
      ctx.accounts.deposit_reward_tokens(), 
      total_reward, 
      reward_mint.unwrap().decimals
    )?;
  }

  Ok(())
}