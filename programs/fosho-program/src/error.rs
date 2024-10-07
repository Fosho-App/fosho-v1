use anchor_lang::prelude::*;

#[error_code]
pub enum FoshoErrors {
  #[msg("Invalid Community Authority")]
  InvalidCommunityAuthority,
  #[msg("Registration end time cannot exceed the event start time")]
  InvalidRegistrationEndTime,
  #[msg("The event must start in a future.")]
  InvalidEventStartTime,
  #[msg("Registration time has been expired")]
  RegistrationTimeExpired,
  #[msg("The maximum allowed attendees have already joined")]
  MaxAttendeesAlreadyJoined,
  #[msg("The rewards cannot be claimed during the pending status")]
  AttendeeStatusPending,
  #[msg("Not a valid claimer")]
  InvalidClaimer,
  #[msg("One of the accounts required for this ix is not provided")]
  AccountNotProvided
}