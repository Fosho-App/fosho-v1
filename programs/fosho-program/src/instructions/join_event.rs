use crate::{
  constant::*,
  error::FoshoErrors,
  state::*,
  utils::{
    create_attribute, create_ticket_plugins, get_capacity_from_attributes,
    get_reg_ends_at_from_attributes, get_reg_starts_at_from_attributes,
  },
};
use anchor_lang::{
  prelude::*,
  system_program::{transfer, Transfer},
};

use mpl_core::{
  accounts::BaseCollectionV1,
  fetch_plugin,
  instructions::CreateV2CpiBuilder,
  types::{Attributes, PluginType},
  ID as MPL_CORE_ID,
};

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
  #[account(
    seeds = [
      COMMUNITY_PRE_SEED.as_ref(),
      community.seed.as_ref(),
    ],
    bump = community.bump,
  )]
  pub community: Box<Account<'info, Community>>,
  #[account(
      mut,
      constraint = event_collection.update_authority == community.key(),
  )]
  pub event_collection: Box<Account<'info, BaseCollectionV1>>,
  /// CHECK: checked against the event authority in the create_event instruction
  /// if it exists they would have to sign this transaction
  pub event_authority: AccountInfo<'info>,
  #[account(mut)]
  pub attendee: Signer<'info>,
  #[account(mut)]
  pub ticket: Signer<'info>,
  pub system_program: Program<'info, System>,
  #[account(address = MPL_CORE_ID)]
  /// CHECK: This is checked by the address constraint
  pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> JoinEvent<'info> {
  pub fn transfer_commitment_fee(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
      from: self.attendee.to_account_info(),
      to: self.event.to_account_info(),
    };

    let cpi_program = self.system_program.to_account_info();

    CpiContext::new(cpi_program, cpi_accounts)
  }

  pub fn create_event_ticket(&self, args: CreateTicketArgs) -> Result<()> {
    // Check that the maximum number of tickets has not been reached yet
    let (_, collection_attribute_list, _) = fetch_plugin::<BaseCollectionV1, Attributes>(
      &self.event_collection.to_account_info(),
      PluginType::Attributes,
    )?;

    let capacity = get_capacity_from_attributes(&collection_attribute_list.attribute_list)?;
    let reg_starts_at =
      get_reg_starts_at_from_attributes(&collection_attribute_list.attribute_list)?;
    let reg_ends_at = get_reg_ends_at_from_attributes(&collection_attribute_list.attribute_list)?;

    if capacity.ne(&0) {
      require!(
        self.event_collection.num_minted < capacity,
        FoshoErrors::MaximumTicketsReached
      );
    }

    let current_unix_ts = Clock::get()?.unix_timestamp as u64;
    if reg_starts_at.ne(&0) {
      require!(
        current_unix_ts >= reg_starts_at,
        FoshoErrors::RegistrationNotStarted
      );
    }

    if reg_ends_at.ne(&0) {
      require!(
        current_unix_ts <= reg_ends_at,
        FoshoErrors::RegistrationEnded
      );
    }

    // Create ticket attributes
    let mut attribute_list = vec![
      create_attribute(
        "Ticket Number",
        (self.event_collection.num_minted + 1).to_string(),
      ),
      create_attribute("Fee", self.event.commitment_fee.to_string()),
    ];

    // Add custom attributes
    if let Some(custom_attrs) = args.custom_attributes {
      for (key, value) in custom_attrs {
        attribute_list.push(create_attribute(&key, value));
      }
    }

    // Create ticket plugins
    let ticket_plugins = create_ticket_plugins(attribute_list, self.community.key());
    let signer_seeds = &[
      COMMUNITY_PRE_SEED.as_ref(),
      self.community.seed.as_ref(),
      &[self.community.bump],
    ];

    // Create the Ticket
    CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
      .asset(&self.ticket.to_account_info())
      .collection(Some(&self.event_collection.to_account_info()))
      .payer(&self.attendee.to_account_info())
      .authority(Some(&self.community.to_account_info()))
      .owner(Some(&self.attendee.to_account_info()))
      .system_program(&self.system_program.to_account_info())
      .name(args.name)
      .uri(args.uri)
      .plugins(ticket_plugins.0)
      .external_plugin_adapters(ticket_plugins.1)
      .invoke_signed(&[signer_seeds])?;

    Ok(())
  }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Default)]
pub struct CreateTicketArgs {
  pub name: String,
  pub uri: String,
  pub custom_attributes: Option<Vec<(String, String)>>,
}

pub fn join_event_handler(ctx: Context<JoinEvent>, args: CreateTicketArgs) -> Result<()> {
  let event = &ctx.accounts.event;

  if event.authority_must_sign {
    let is_community_authority = ctx.accounts.event_authority.key() == ctx.accounts.community.authority;
    if !is_community_authority {
      require!(
        event
          .event_authorities
          .contains(&ctx.accounts.event_authority.key()),
        FoshoErrors::InvalidEventAuthority
      );
    }
    require!(
      ctx.accounts.event_authority.is_signer,
      FoshoErrors::EventAuthorityMustSign
    );
  }

  if event.is_cancelled {
    return Err(FoshoErrors::EventCancelled.into());
  }

  // handled by event collection
  // let clock = Clock::get().unwrap();
  // let current_time = clock.unix_timestamp;
  // require_gte!(event.registration_end_time, current_time, FoshoErrors::RegistrationTimeExpired);
  // require_gt!(event.max_attendees, event.current_attendees, FoshoErrors::MaxAttendeesAlreadyJoined);

  ctx.accounts.create_event_ticket(args)?;

  if event.commitment_fee.gt(&0) {
    transfer(ctx.accounts.transfer_commitment_fee(), event.commitment_fee)?;
  }

  // handled by event collection
  // let event = &mut ctx.accounts.event;
  // event.current_attendees = event.current_attendees.checked_add(1).unwrap();

  // data used for the claiming of rewards
  let attendee_record = &mut ctx.accounts.attendee_record;
  attendee_record.owner = ctx.accounts.attendee.key();
  attendee_record.event = ctx.accounts.event.key();
  attendee_record.status = AttendeeStatus::Pending;
  attendee_record.bump = ctx.bumps.attendee_record;

  Ok(())
}
