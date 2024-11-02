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
}
