use crate::{
  constant::*,
  error::FoshoErrors,
  state::*,
  utils::{
    assert_is_ata, create_attribute, create_ticket_plugins, get_capacity_from_attributes,
    get_reg_ends_at_from_attributes, get_reg_starts_at_from_attributes, get_spl_token_amount,
    validate_nft_collection, validate_verified_nft_creator,
  },
};
use anchor_lang::{
  prelude::*,
  system_program::{transfer, Transfer},
};

use anchor_spl::token_interface::TokenInterface;
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
  pub attendee_record: Box<Account<'info, Attendee>>,
  #[account(
    mut,
    seeds = [
      EVENT_PRE_SEED.as_ref(),
      event.community.key().as_ref(),
      &event.nonce.to_le_bytes()
    ],
    bump = event.bump,
  )]
  pub event: Box<Account<'info, Event>>,
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
      seeds = [
        EVENT_PRE_SEED.as_ref(),
        event.key().as_ref(),
        EVENT_COLLECTION_SUFFIX_SEED.as_ref(),
      ],
      bump,
      constraint = event_collection.update_authority == community.key(),
  )]
  pub event_collection: Box<Account<'info, BaseCollectionV1>>,
  /// CHECK: checked against the event authority in the create_event instruction
  /// if it exists they would have to sign this transaction
  pub event_authority: AccountInfo<'info>,
  #[account(mut)]
  pub attendee: Signer<'info>,
  /// CHECK: safe because the ticket is created in this instruction
  #[account(mut,
    seeds = [
      EVENT_PRE_SEED.as_ref(),
      event.key().as_ref(),
      attendee.key().as_ref(),
      TICKET_SUFFIX_SEED.as_ref(),
    ],
    bump)]
  pub ticket: UncheckedAccount<'info>,
  pub system_program: Program<'info, System>,
  #[account(address = MPL_CORE_ID)]
  /// CHECK: This is checked by the address constraint
  pub mpl_core_program: UncheckedAccount<'info>,
  pub token_program: Interface<'info, TokenInterface>,
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

  pub fn create_event_ticket(&self, ticket_bump: u8) -> Result<()> {
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
    let attribute_list = vec![
      create_attribute(
        "Ticket Number",
        (self.event_collection.num_minted + 1).to_string(),
      ),
      create_attribute("Fee", self.event.commitment_fee.to_string()),
    ];

    // Create ticket plugins
    let ticket_plugins = create_ticket_plugins(attribute_list, self.community.key());
    let signer_seeds = &[
      COMMUNITY_PRE_SEED.as_ref(),
      self.community.seed.as_ref(),
      &[self.community.bump],
    ];

    let event_binding = self.event.key();
    let attendee_binding = self.attendee.key();
    let ticket_seeds = &[
      EVENT_PRE_SEED.as_ref(),
      event_binding.as_ref(),
      attendee_binding.as_ref(),
      TICKET_SUFFIX_SEED.as_ref(),
      &[ticket_bump],
    ];

    // we derive the name from the collection but add Ticket + No.
    let name = format!(
      "{} #{}",
      self.event_collection.name,
      self.event_collection.num_minted + 1
    );
    let uri = self.event_collection.uri.clone();
    // Create the Ticket
    CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
      .asset(&self.ticket.to_account_info())
      .collection(Some(&self.event_collection.to_account_info()))
      .payer(&self.attendee.to_account_info())
      .authority(Some(&self.community.to_account_info()))
      .owner(Some(&self.attendee.to_account_info()))
      .system_program(&self.system_program.to_account_info())
      .name(name)
      .uri(uri)
      .plugins(ticket_plugins.0)
      .external_plugin_adapters(ticket_plugins.1)
      .invoke_signed(&[signer_seeds, ticket_seeds])?;

    Ok(())
  }

  pub fn validate_event_version<'a>(&self, remaining_accounts: &[AccountInfo<'a>]) -> Result<()> {
    let remaining_account_iter = &mut remaining_accounts.iter();

    match &self.event.event_version {
      EventVersion::NftGated(nft_data) => {
        let mut validation_results = vec![];
        // Check if we have enough remaining accounts
        if remaining_accounts.len() < 3 {
          return Err(FoshoErrors::NotEnoughRemainingAccounts.into());
        }
        let mint_account = next_account_info(remaining_account_iter)?;
        let mint_account_key = mint_account.key();
        let ata_mint_account = next_account_info(remaining_account_iter)?;
        let mint_metadata_account = next_account_info(remaining_account_iter)?;

        assert_is_ata(
          ata_mint_account,
          &self.attendee.key(),
          &mint_account_key,
          true,
          &self.token_program.key(),
        )?;
        if let Some(verified_creator) = nft_data.verified_creator {
          validation_results
            .push(validate_verified_nft_creator(mint_metadata_account, &verified_creator).is_ok());
        }
        if let Some(collection_mint) = nft_data.collection_mint {
          validation_results
            .push(validate_nft_collection(mint_metadata_account, collection_mint).is_ok());
        }
        if !validation_results.iter().any(|&x| x) {
          return Err(FoshoErrors::InvalidCollectionDetails.into());
        }
      }
      EventVersion::TokenGated(token_data) => {
        let mut validation_results = vec![];
        // Check if we have enough remaining accounts
        if remaining_accounts.len() < 2 {
          return Err(FoshoErrors::NotEnoughRemainingAccounts.into());
        }

        let mint_account = next_account_info(remaining_account_iter)?;
        let mint_account_key = mint_account.key();
        let ata_mint_account = next_account_info(remaining_account_iter)?;

        assert_is_ata(
          ata_mint_account,
          &self.attendee.key(),
          &mint_account_key,
          true,
          &self.token_program.key(),
        )?;

        if let Some(mint) = token_data.mint {
          validation_results.push(mint == mint_account_key);
        }
        if let Some(min_amount) = token_data.minimum_amount {
          let owned_amount = get_spl_token_amount(&ata_mint_account)?;
          validation_results.push(owned_amount >= min_amount);
        }

        if !validation_results.iter().any(|&x| x) {
          return Err(FoshoErrors::InvalidTokenDetails.into());
        }
      }
      _ => {} // Regular events always pass the validation
    }

    Ok(())
  }
}

pub fn join_event_handler(ctx: Context<JoinEvent>) -> Result<()> {
  let event = &ctx.accounts.event;

  if event.authority_must_sign {
    let is_community_authority =
      ctx.accounts.event_authority.key() == ctx.accounts.community.authority;
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

  ctx.accounts.create_event_ticket(ctx.bumps.ticket)?;

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

  match event.event_version {
    EventVersion::Regular => {}
    _ => {
      ctx
        .accounts
        .validate_event_version(&ctx.remaining_accounts)?;
    }
  }
  Ok(())
}
