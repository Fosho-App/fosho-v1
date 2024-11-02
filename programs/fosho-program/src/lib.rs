use anchor_lang::prelude::*;

declare_id!("DQzCnhf6qTaz2tPPj6jvicntC9hP2tqDzZp1RWKujXdT");

mod instructions;
use instructions::*;
pub mod constant;
pub mod error;
pub mod state;
pub mod utils;

#[program]
pub mod fosho_program {
  use super::*;

  pub fn create_community(
    ctx: Context<CreateCommunity>,
    seed: Pubkey,
    community_name: String,
  ) -> Result<()> {
    log_version();
    create_community_handler(ctx, seed, community_name)
  }

  pub fn create_event(
    ctx: Context<CreateEvent>,
    args: CreateEventArgs,
    reward_per_user: u64,
    // event_authorities can sign join_event ixn
    // and the verify_attendance ixn
    event_authorities: Vec<Pubkey>,
    // authorities must sign join_event ixn
    authority_must_sign: bool,
  ) -> Result<()> {
    log_version();
    create_event_handler(
      ctx,
      args,
      reward_per_user,
      event_authorities,
      authority_must_sign,
    )
  }

  pub fn join_event(ctx: Context<JoinEvent>, args: CreateTicketArgs) -> Result<()> {
    log_version();
    join_event_handler(ctx, args)
  }

  pub fn verify_attendee(ctx: Context<VerifyAttendee>) -> Result<()> {
    log_version();
    verify_attendee_handler(ctx)
  }

  pub fn reject_attendee(ctx: Context<RejectAttendee>) -> Result<()> {
    log_version();
    reject_attendee_handler(ctx)
  }

  pub fn cancel_event(ctx: Context<CancelEvent>) -> Result<()> {
    log_version();
    cancel_event_handler(ctx)
  }

  pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    log_version();
    claim_rewards_handler(ctx)
  }
}

fn log_version() {
  msg!("VERSION:{:?}", env!("CARGO_PKG_VERSION"));
}
