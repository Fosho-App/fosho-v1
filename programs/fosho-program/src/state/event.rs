use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Event {
  pub community: Pubkey,
  pub reward_mint: Option<Pubkey>,
  pub event_version: EventVersion,
  /// 4 event authorities are allowed.
  #[max_len(4)]
  pub event_authorities: Vec<Pubkey>,
  pub commitment_fee: u64,
  pub bump: u8,
  pub nonce: u32,
  pub reward_per_user: u64,
  pub is_cancelled: bool,
  /// in all cases, event authority must sign the attendance.
  /// if this is true. event authority must sign the join event instruction.
  pub authority_must_sign: bool,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, PartialEq, Eq, Debug)]
pub enum EventType {
  InPerson,
  Virtual,
  Exhibition,
  Conference,
  Concert,
  SportingEvent,
  Workshop,
  Webinar,
  NetworkingEvent,
  Other(String),
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, PartialEq, Eq, Debug, InitSpace)]
pub struct NftData {
  /// mpl token metadata collection
  pub collection_mint: Option<Pubkey>,
  /// mpl token metadata verified creator pubkey
  pub verified_creator: Option<Pubkey>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, PartialEq, Eq, Debug, InitSpace)]
pub struct TokenData {
  /// token mint of the token that must be held by the owner
  pub mint: Option<Pubkey>,
  /// minimum amount held by the owner
  pub minimum_amount: Option<u64>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, PartialEq, Eq, Debug, InitSpace)]
pub enum EventVersion {
  Regular,
  NftGated(NftData),
  TokenGated(TokenData),
}
