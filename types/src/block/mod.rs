// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Core data types for blocks in the tangle.

#[macro_use]
mod r#macro;
mod block_id;
mod error;

/// A module that provides DTOs.
#[cfg(feature = "dto")]
pub mod dto;

/// A module that provides types and syntactic validations of addresses.
pub mod address;
/// A module that provides types and syntactic validations of blocks.
#[allow(clippy::module_inception)]
pub mod block;
/// A module that contains helper functions and types.
pub mod helper;
/// A module that provides types and syntactic validations of inputs.
pub mod input;
/// A module that provides types and syntactic validations of outputs.
pub mod output;
/// A module that provides types and syntactic validations of parents.
pub mod parent;
/// A module that provides types and syntactic validations of payloads.
pub mod payload;
/// A module that provides types and syntactic validations of protocol parameters.
pub mod protocol;
/// A module that provides utilities for random generation of types.
#[cfg(feature = "rand")]
pub mod rand;
/// A module that provides types and rules for semantic validation.
pub mod semantic;
/// A module that provides types and syntactic validations of signatures.
pub mod signature;
/// A module that provides types and syntactic validations of unlocks.
pub mod unlock;

#[cfg(feature = "inx")]
pub use self::error::inx::InxError;
#[cfg(feature = "dto")]
pub use self::{block::dto::BlockDto, error::dto::DtoError};
pub use self::{
    block::{Block, BlockBuilder},
    block_id::BlockId,
    error::Error,
};

pub(crate) const PROTOCOL_VERSION: u8 = 2;
