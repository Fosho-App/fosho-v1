use anchor_lang::prelude::*;

use mpl_core::types::{
  AppDataInitInfo, Attribute, Attributes, ExternalPluginAdapterInitInfo,
  ExternalPluginAdapterSchema, PermanentBurnDelegate, PermanentFreezeDelegate,
  PermanentTransferDelegate, Plugin, PluginAuthority, PluginAuthorityPair,
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
