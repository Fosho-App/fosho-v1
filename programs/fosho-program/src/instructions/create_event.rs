use crate::{constant::*, error::FoshoErrors, state::*, utils::create_attribute};
use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use mpl_core::{
  instructions::CreateCollectionV2CpiBuilder,
  types::{Attributes, Plugin, PluginAuthority, PluginAuthorityPair},
  ID as MPL_CORE_ID,
};

#[derive(Accounts)]
#[instruction()]
pub struct CreateEvent<'info> {
  #[account(
    init,
    seeds = [
      EVENT_PRE_SEED.as_ref(),
      community.key().as_ref(),
      &community.events_count.to_le_bytes()
    ],
    bump,
    payer = authority,
    space = 8 + Event::INIT_SPACE
  )]
  pub event: Account<'info, Event>,
  /// CHECK: event_collection is created.
  #[account(mut,
    seeds = [
      EVENT_PRE_SEED.as_ref(),
      event.key().as_ref(),
      EVENT_COLLECTION_SUFFIX_SEED.as_ref(),
    ],
    bump,)]
  pub event_collection: UncheckedAccount<'info>,
  #[account(
    mut,
    seeds = [
      COMMUNITY_PRE_SEED.as_ref(),
      community.seed.as_ref(),
    ],
    bump = community.bump,
    has_one = authority
  )]
  pub community: Box<Account<'info, Community>>,
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
  pub system_program: Program<'info, System>,
  /// CHECK: This is checked by the address constraint
  #[account(address = MPL_CORE_ID)]
  pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> CreateEvent<'info> {
  pub fn deposit_reward_tokens(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
    let cpi_accounts = TransferChecked {
      from: self.sender_account.as_ref().unwrap().to_account_info(),
      to: self.reward_account.as_ref().unwrap().to_account_info(),
      mint: self.reward_mint.as_ref().unwrap().to_account_info(),
      authority: self.authority.to_account_info(),
    };

    let cpi_program = self.token_program.to_account_info();

    CpiContext::new(cpi_program, cpi_accounts)
  }

  pub fn create_event_collection(
    &self,
    name: String,
    uri: String,
    event_type: EventType,
    organizer: String,
    commitment_fee: u64,
    event_starts_at: Option<i64>,
    event_ends_at: Option<i64>,
    registration_starts_at: Option<i64>,
    registration_ends_at: Option<i64>,
    capacity: Option<u64>,
    location: Option<String>,
    virtual_link: Option<String>,
    description: Option<String>,
    event_collection_bump: u8,
  ) -> Result<()> {
    let mut attribute_list = vec![
      create_attribute("Event Type", format!("{:?}", event_type)),
      create_attribute("Organizer", organizer.to_string()),
      create_attribute("Fee", commitment_fee.to_string()),
    ];

    macro_rules! add_optional_attribute {
      ($key:expr, $value:expr) => {
        if let Some(value) = $value {
          attribute_list.push(create_attribute($key, value.to_string()));
        }
      };
    }

    let event_start_time = if event_starts_at.is_some() {
      let event_start_time_unwraped = event_starts_at.unwrap();
      let clock = Clock::get().unwrap();
      let current_time = clock.unix_timestamp;

      require_gt!(
        event_start_time_unwraped,
        current_time,
        FoshoErrors::InvalidEventStartTime
      );
      event_start_time_unwraped
    } else {
      0
    };

    if let Some(reg_end_time) = registration_ends_at {
      if event_start_time.gt(&reg_end_time) {
        return Err(FoshoErrors::InvalidRegistrationEndTime.into());
      }
    };

    add_optional_attribute!("Event Starts At", Some(event_start_time as u64));
    add_optional_attribute!("Event Ends At", event_ends_at);
    add_optional_attribute!("Registration Starts At", registration_starts_at);
    add_optional_attribute!("Registration Ends At", registration_ends_at);
    add_optional_attribute!("Capacity", capacity);
    add_optional_attribute!("Location", location);
    add_optional_attribute!("Virtual Link", virtual_link);
    add_optional_attribute!("Description", description);

    let collection_plugin = vec![PluginAuthorityPair {
      plugin: Plugin::Attributes(Attributes { attribute_list }),
      authority: Some(PluginAuthority::UpdateAuthority),
    }];
    let event_binding = self.event.key();
    let event_collection_seeds = &[
      EVENT_PRE_SEED.as_ref(),
      event_binding.as_ref(),
      EVENT_COLLECTION_SUFFIX_SEED.as_ref(),
      &[event_collection_bump],
    ];
    // Create the Event Collection
    CreateCollectionV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
      .collection(&self.event_collection.to_account_info())
      .update_authority(Some(&self.community.to_account_info()))
      .payer(&self.authority.to_account_info())
      .system_program(&self.system_program.to_account_info())
      .name(name.clone())
      .uri(uri.clone())
      .plugins(collection_plugin)
      .invoke_signed(&[event_collection_seeds])?;

    Ok(())
  }
}

pub fn create_event_handler(
  ctx: Context<CreateEvent>,
  name: String,
  uri: String,
  event_type: EventType,
  organizer: String,
  commitment_fee: u64,
  event_starts_at: Option<i64>,
  event_ends_at: Option<i64>,
  registration_starts_at: Option<i64>,
  registration_ends_at: Option<i64>,
  capacity: Option<u64>,
  location: Option<String>,
  virtual_link: Option<String>,
  description: Option<String>,
  reward_per_user: u64,
  // event_authorities can sign join_event ixn
  // and the verify_attendance ixn
  event_authorities: Vec<Pubkey>,
  // authorities must sign join_event ixn
  authority_must_sign: bool,
) -> Result<()> {
  let event = &mut ctx.accounts.event;
  let community = &ctx.accounts.community;

  let reward_mint = ctx.accounts.reward_mint.as_ref();

  event.reward_mint = if let Some(reward_mint_acc) = reward_mint {
    Some(reward_mint_acc.key())
  } else {
    None
  };
  let max_attendees = capacity.unwrap_or(1);
  event.commitment_fee = commitment_fee;

  // handled by the event collection
  // event.max_attendees = max_attendees as u32;
  // event.event_start_time = event_start_time;

  event.community = community.key();
  event.nonce = ctx.accounts.community.events_count;
  event.bump = ctx.bumps.event;
  event.is_cancelled = false;
  event.reward_per_user = reward_per_user;
  event.authority_must_sign = authority_must_sign;
  event.event_authorities = event_authorities;

  let community_mut = &mut ctx.accounts.community;
  community_mut.events_count += 1;

  if reward_per_user.gt(&0) {
    if ctx.accounts.reward_mint.is_none()
      || ctx.accounts.reward_account.is_none()
      || ctx.accounts.sender_account.is_none()
    {
      return Err(FoshoErrors::AccountNotProvided.into());
    }

    let total_reward = reward_per_user.checked_mul(max_attendees as u64).unwrap();

    transfer_checked(
      ctx.accounts.deposit_reward_tokens(),
      total_reward,
      reward_mint.unwrap().decimals,
    )?;
  }
  // creates the event collection
  ctx.accounts.create_event_collection(
    name,
    uri,
    event_type,
    organizer,
    commitment_fee,
    event_starts_at,
    event_ends_at,
    registration_starts_at,
    registration_ends_at,
    capacity,
    location,
    virtual_link,
    description,
    ctx.bumps.event_collection,
  )?;

  Ok(())
}
