// deny (not forbid): env mutation helpers require a scoped unsafe allowance.
#![deny(unsafe_code)]
//! IAOP shared test infrastructure.
//!
//! Framework-light helpers usable from any service's test suite:
//! unique identifiers, env guards, JSON assertions, and event-envelope
//! builders matching `iaop-event-contracts`.
//!
//! This crate intentionally contains NO domain fixtures or business
//! semantics — those belong to each service's own test suite.

pub mod concurrency;
pub mod env;
pub mod events;
pub mod ids;
pub mod json_assert;
pub mod kafka;
pub mod mock_provider;
pub mod oidc;
pub mod postgres;
