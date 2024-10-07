use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use crate::{constant::*, error::FoshoErrors, state::*};

#[derive(Accounts)]
pub struct JoinEvent<'info> {
  #[account(
    init,
    payer = attendee,
    space = 8 + Attendee::ATTENDEE_SIZE,
    seeds = [
      ATTENDEE_PRE_SEED.as_ref(),
      event.key().as_ref(),
      attendee.key().as_ref()
    ],
    bump,
  )]
  pub attendee_record: Account<'info, Attendee>,
  #[account(
    mut,
    seeds = [
      EVENT_PRE_SEED.as_ref(),
      event.community.key().as_ref(),
      &event.nonce.to_le_bytes()
    ],
    bump = event.bump,
  )]
  pub event: Account<'info, Event>,
  #[account(mut)]
  pub attendee: Signer<'info>,
  pub system_program: Program<'info, System>
}

impl<'info> JoinEvent<'info> {
  pub fn transfer_commitment_fee(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
      from: self.attendee.to_account_info(),
      to: self.event.to_account_info()
    };

    let cpi_program = self.system_program.to_account_info();

    CpiContext::new(cpi_program, cpi_accounts)
  }
}

pub fn join_event_handler(
  ctx: Context<JoinEvent>
) -> Result<()> {
  let event = &ctx.accounts.event;

  let clock = Clock::get().unwrap();
  let current_time = clock.unix_timestamp;

  require_gte!(event.registration_end_time, current_time, FoshoErrors::RegistrationTimeExpired);
  require_gt!(event.max_attendees, event.current_attendees, FoshoErrors::MaxAttendeesAlreadyJoined);

  if event.commitment_fee.gt(&0) {
    transfer(ctx.accounts.transfer_commitment_fee(), event.commitment_fee)?;
  }

  let event = &mut ctx.accounts.event;
  event.current_attendees = event.current_attendees.checked_add(1).unwrap();

  let attendee_record = &mut ctx.accounts.attendee_record;
  attendee_record.owner = ctx.accounts.attendee.key();
  attendee_record.event = ctx.accounts.event.key();
  attendee_record.status = AttendeeStatus::Pending;
  attendee_record.bump = ctx.bumps.attendee_record;

  Ok(())
}