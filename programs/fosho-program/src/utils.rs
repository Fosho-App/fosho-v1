use anchor_lang::{
  prelude::*,
  solana_program::{program_memory::sol_memcmp, pubkey::PUBKEY_BYTES},
};

use anchor_spl::{
  associated_token::get_associated_token_address_with_program_id,
  metadata::MetadataAccount,
  token_2022::spl_token_2022::{extension::StateWithExtensions, state::Account},
};
use arrayref::array_ref;
use mpl_core::{
  accounts::BaseAssetV1,
  fetch_external_plugin_adapter_data_info,
  types::{
    AppDataInitInfo, Attribute, Attributes, ExternalPluginAdapterInitInfo,
    ExternalPluginAdapterKey, ExternalPluginAdapterSchema, PermanentBurnDelegate,
    PermanentFreezeDelegate, PermanentTransferDelegate, Plugin, PluginAuthority,
    PluginAuthorityPair,
  },
};

use crate::error::FoshoErrors;

pub fn create_attribute<K: Into<String>, V: Into<String>>(key: K, value: V) -> Attribute {
  Attribute {
    key: key.into(),
    value: value.into(),
  }
}

pub fn get_capacity_from_attributes(attribute_list: &[Attribute]) -> Result<u32> {
  let capacity_attribute = attribute_list.iter().find(|attr| attr.key == "Capacity");
  match capacity_attribute {
    Some(capacity_attribute) => capacity_attribute
      .value
      .parse::<u32>()
      .map_err(|_| FoshoErrors::NumericalOverflow.into()),
    None => return Ok(0),
  }
}

pub fn get_reg_starts_at_from_attributes(attribute_list: &[Attribute]) -> Result<u64> {
  let reg_starts_at = attribute_list
    .iter()
    .find(|attr| attr.key == "Registration Starts At");
  match reg_starts_at {
    Some(reg_starts_at) => reg_starts_at
      .value
      .parse::<u64>()
      .map_err(|_| FoshoErrors::NumericalOverflow.into()),
    None => return Ok(0),
  }
}

pub fn get_reg_ends_at_from_attributes(attribute_list: &[Attribute]) -> Result<u64> {
  let reg_ends_at = attribute_list
    .iter()
    .find(|attr| attr.key == "Registration Ends At");
  match reg_ends_at {
    Some(reg_ends_at) => reg_ends_at
      .value
      .parse::<u64>()
      .map_err(|_| FoshoErrors::NumericalOverflow.into()),
    None => return Ok(0),
  }
}

pub fn get_event_ends_at_from_attributes(attribute_list: &[Attribute]) -> Result<u64> {
  let event_ends_at = attribute_list
    .iter()
    .find(|attr| attr.key == "Event Ends At");
  match event_ends_at {
    Some(event_ends_at) => event_ends_at
      .value
      .parse::<u64>()
      .map_err(|_| FoshoErrors::NumericalOverflow.into()),
    None => return Ok(0),
  }
}

pub fn get_event_starts_at_from_attributes(attribute_list: &[Attribute]) -> Result<u64> {
  let event_starts_at = attribute_list
    .iter()
    .find(|attr| attr.key == "Event Starts At");
  match event_starts_at {
    Some(event_starts_at) => event_starts_at
      .value
      .parse::<u64>()
      .map_err(|_| FoshoErrors::NumericalOverflow.into()),
    None => return Ok(0),
  }
}

pub fn create_ticket_plugins(
  attributes: Vec<Attribute>,
  event_authority: Pubkey,
) -> (Vec<PluginAuthorityPair>, Vec<ExternalPluginAdapterInitInfo>) {
  let plugins = vec![
    PluginAuthorityPair {
      plugin: Plugin::Attributes(Attributes {
        attribute_list: attributes,
      }),
      authority: Some(PluginAuthority::UpdateAuthority),
    },
    PluginAuthorityPair {
      plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
      authority: Some(PluginAuthority::UpdateAuthority),
    },
    PluginAuthorityPair {
      plugin: Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
      authority: Some(PluginAuthority::UpdateAuthority),
    },
    PluginAuthorityPair {
      plugin: Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
      authority: Some(PluginAuthority::UpdateAuthority),
    },
  ];

  let external_plugins = vec![ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
    init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
    data_authority: PluginAuthority::Address {
      address: event_authority,
    },
    schema: Some(ExternalPluginAdapterSchema::Binary),
  })];

  (plugins, external_plugins)
}

pub fn check_if_already_scanned<'a>(ticket: AccountInfo<'a>, authority: &Pubkey) -> Result<()> {
  let app_data_length = match fetch_external_plugin_adapter_data_info::<BaseAssetV1>(
    &ticket,
    None,
    &ExternalPluginAdapterKey::AppData(PluginAuthority::Address {
      address: authority.key(),
    }),
  ) {
    Ok((_, length)) => length,
    Err(_) => 0,
  };

  Ok(require!(app_data_length == 0, FoshoErrors::AlreadyScanned))
}

// Custom function to validate nft belongs to a specified collection and is verified
pub fn validate_nft_collection(nft_metadata: &AccountInfo, collection_mint: Pubkey) -> Result<()> {
  let metadata_data = nft_metadata.try_borrow_data()?;
  let data = &mut metadata_data.as_ref();
  let metadata = MetadataAccount::try_deserialize(data)?;

  let collection = &metadata.collection;
  let collection_details = &metadata.collection_details;

  if collection_details.is_some() {
    return Err(FoshoErrors::InvalidCollectionDetails.into());
  }

  match collection {
    Some(collection) => {
      if collection.key != collection_mint {
        return Err(FoshoErrors::InvalidCollection.into());
      }
      if !collection.verified {
        return Err(FoshoErrors::NftNotVerified.into());
      }
    }
    None => return Err(FoshoErrors::InvalidCollection.into()),
  }

  Ok(())
}

// Custom function to validate nft belongs to a specified creator and is verified
pub fn validate_verified_nft_creator(
  nft_metadata: &AccountInfo,
  verified_nft_creator: &Pubkey,
) -> Result<()> {
  let metadata_data = nft_metadata.try_borrow_data()?;
  let data = &mut metadata_data.as_ref();
  let metadata = MetadataAccount::try_deserialize(data)?;
  if let Some(creators) = &metadata.creators {
    for creator in creators {
      if creator.address == *verified_nft_creator && creator.verified {
        return Ok(());
      }
    }
    return Err(FoshoErrors::InvalidCreator.into());
  } else {
    return Err(FoshoErrors::NoCreatorsPresentOnMetadata.into());
  }
}

pub fn assert_is_ata(
  ata: &AccountInfo,
  wallet: &Pubkey,
  mint: &Pubkey,
  initialized: bool,
  token_program: &Pubkey,
) -> Result<()> {
  if initialized {
    let ata_data = ata.data.borrow();
    let ata_account = StateWithExtensions::<Account>::unpack(&ata_data)?;

    assert_owned_by(ata, &token_program)?;
    assert_keys_equal(ata_account.base.owner, *wallet)?;
    assert_keys_equal(ata_account.base.mint, *mint)?;
  }
  assert_keys_equal(
    get_associated_token_address_with_program_id(wallet, mint, token_program),
    *ata.key,
  )?;
  Ok(())
}

pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> Result<()> {
  if sol_memcmp(key1.as_ref(), key2.as_ref(), PUBKEY_BYTES) != 0 {
    msg!("Wrong public key: {} should be {}", key1, key2);
    return err!(FoshoErrors::PublicKeyMismatch);
  } else {
    Ok(())
  }
}

// cheapest pubkey comparing
pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
  sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<()> {
  if account.owner != owner {
    msg!("Wrong account owner: {} should be {}", account.owner, owner);
    return Err(FoshoErrors::WrongAccountOwner.into());
  }
  Ok(())
}

/// Computationally cheap method to get amount from a token account
/// It reads amount without deserializing full account data
pub fn get_spl_token_amount(token_account_info: &AccountInfo) -> Result<u64> {
  // TokeAccount layout:   mint(32), owner(32), amount(8), ...
  let data = token_account_info.try_borrow_data()?;
  let amount_bytes = array_ref![data, 64, 8];

  Ok(u64::from_le_bytes(*amount_bytes))
}
