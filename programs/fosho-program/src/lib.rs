use anchor_lang::prelude::*;

declare_id!("5ojhS89XpkvrSyagCddG7fv4wh2ffx8kVA6ABDYttZdN");

mod instructions;
use instructions::*;
pub mod state;
pub mod error;
pub mod constant;

#[program]
pub mod fosho_program {
    use super::*;

    pub fn create_community(ctx: Context<CreateCommunity>, seed: Pubkey) -> Result<()> {
        log_version();
        create_community_handler(ctx, seed)
    }

    pub fn create_event(
        ctx: Context<CreateEvent>, 
        nonce: u32,
        max_attendees: u32,
        commitment_fee: u64,
        event_start_time: i64,
        registration_end_time: Option<i64>,
        reward_per_user: u64
    ) -> Result<()> {
        log_version();
        create_event_handler(
            ctx, nonce, max_attendees, commitment_fee, event_start_time, registration_end_time, reward_per_user
        )
    }

    pub fn join_event(ctx: Context<JoinEvent>) -> Result<()> {
        log_version();
        join_event_handler(ctx)
    }

    pub fn verify_attendee(ctx: Context<VerifyAttendee>) -> Result<()> {
        log_version();
        verify_attendee_handler(ctx)
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
