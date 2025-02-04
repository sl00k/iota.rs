// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{InputSelection, Requirement};
use crate::{
    block::address::Address,
    error::{Error, Result},
    secret::types::InputSigningData,
};

impl InputSelection {
    /// Fulfills an issuer requirement by fulfilling the equivalent sender requirement.
    /// Potentially converts the error for a more accurate one.
    pub(crate) fn fulfill_issuer_requirement(
        &mut self,
        address: Address,
    ) -> Result<(Vec<InputSigningData>, Option<Requirement>)> {
        match self.fulfill_sender_requirement(address) {
            Ok(res) => Ok(res),
            Err(Error::UnfulfillableRequirement(Requirement::Sender(_))) => {
                Err(Error::UnfulfillableRequirement(Requirement::Issuer(address)))
            }
            Err(e) => Err(e),
        }
    }
}
