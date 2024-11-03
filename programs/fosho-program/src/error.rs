use anchor_lang::prelude::*;

#[error_code]
pub enum FoshoErrors {
  #[msg("Invalid Community Authority")]
  InvalidCommunityAuthority,
  #[msg("Registration end time cannot exceed the event start time")]
  InvalidRegistrationEndTime,
  #[msg("The event must start in a future.")]
  InvalidEventStartTime,
  #[msg("The registration period has not started yet")]
  RegistrationNotStarted,
  #[msg("The registration period has ended")]
  RegistrationEnded,
  #[msg("The maximum number of tickets has been reached")]
  MaximumTicketsReached,
  #[msg("The rewards cannot be claimed during the pending status")]
  AttendeeStatusPending,
  #[msg("Not a valid claimer")]
  InvalidClaimer,
  #[msg("One of the accounts required for this ix is not provided")]
  AccountNotProvided,
  #[msg("this attendee has already claimed the rewards")]
  AlreadyClaimed,
  #[msg("The attribute is missing")]
  MissingAttribute,
  #[msg("Numerical Overflow")]
  NumericalOverflow,
  #[msg("Event Authority must sign")]
  EventAuthorityMustSign,
  #[msg("Event Authority Publickey mismatch")]
  InvalidEventAuthority,
  #[msg("Ticket has been signed already")]
  AlreadyScanned,
  #[msg("Event is cancelled")]
  EventCancelled,
  #[msg("Event has not ended")]
  EventHasNotEnded,
  #[msg("Event has not started")]
  EventHasNotStarted,
  #[msg("Event has ended")]
  EventEnded,
  #[msg("Invalid Collection")]
  InvalidCollection,
  #[msg("Invalid Collection Details")]
  InvalidCollectionDetails,
  #[msg("Nft Not Verified")]
  NftNotVerified,
  #[msg("Collection Key is Missing")]
  CollectionMissing,
  #[msg("A verified creator is missing")]
  VerifiedCreatorMissing,
  #[msg("Invalid nft creator")]
  InvalidCreator,
  #[msg("No creators on metadata")]
  NoCreatorsPresentOnMetadata,
  #[msg("Public Key mismatch")]
  PublicKeyMismatch,
  #[msg("Incorrect account owner")]
  WrongAccountOwner,
  #[msg("Not enough remaining accounts provided")]
  NotEnoughRemainingAccounts,
  #[msg("User may not have enough tokens or incorrect data has been supplied")]
  InvalidTokenDetails,
}
