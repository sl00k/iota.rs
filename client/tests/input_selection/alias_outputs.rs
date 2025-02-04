// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_client::{
    api::input_selection::{Burn, InputSelection, Requirement},
    block::{
        address::Address,
        output::{AliasId, AliasOutputBuilder, Output},
        protocol::protocol_parameters,
    },
    Error,
};

use crate::input_selection::{
    build_inputs, build_outputs, is_remainder_or_return, unsorted_eq,
    Build::{Alias, Basic},
    ALIAS_ID_0, ALIAS_ID_1, ALIAS_ID_2, BECH32_ADDRESS_ALIAS_1, BECH32_ADDRESS_ED25519_0, BECH32_ADDRESS_ED25519_1,
    BECH32_ADDRESS_NFT_1, TOKEN_SUPPLY,
};

#[test]
fn input_alias_eq_output_alias() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert!(unsorted_eq(&selected.outputs, &outputs));
}

#[test]
fn transition_alias_id_zero() {
    let protocol_parameters = protocol_parameters();
    let alias_id_0 = AliasId::from_str(ALIAS_ID_0).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_000,
        alias_id_0,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let alias_id = AliasId::from(inputs[0].output_id());
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert!(unsorted_eq(&selected.outputs, &outputs));
}

#[test]
fn input_amount_lt_output_amount() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Basic(2_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::InsufficientAmount {
            found: 1_000_000,
            // Amount we want to send + storage deposit for alias remainder
            required: 2_251_500,
        })
    ));
}

#[test]
fn input_amount_lt_output_amount_2() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![
        Alias(2_000_000, alias_id_2, BECH32_ADDRESS_ED25519_0, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let outputs = build_outputs(vec![Basic(3_000_001, BECH32_ADDRESS_ED25519_0, None, None, None)]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::InsufficientAmount {
            found: 3_000_000,
            // Amount we want to send + storage deposit for alias remainder
            required: 3_251_501
        })
    ));
}

#[test]
fn basic_output_with_alias_input() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        2_251_500,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Basic(2_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);

    let selected = InputSelection::new(inputs.clone(), outputs, protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    // basic output + alias remainder
    assert_eq!(selected.outputs.len(), 2);
}

#[test]
fn create_alias() {
    let protocol_parameters = protocol_parameters();
    let alias_id_0 = AliasId::from_str(ALIAS_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(2_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_0,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);

    let selected = InputSelection::new(inputs.clone(), outputs, protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    // One output should be added for the remainder
    assert_eq!(selected.outputs.len(), 2);
    // Output contains the new minted alias id
    assert!(selected.outputs.iter().any(|output| {
        if let Output::Alias(alias_output) = output {
            *alias_output.alias_id() == alias_id_0
        } else {
            false
        }
    }));
}

#[test]
fn burn_alias() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        2_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Basic(2_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .burn(Burn::new().add_alias(alias_id_2))
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert!(unsorted_eq(&selected.outputs, &outputs));
}

#[test]
fn not_enough_storage_deposit_for_remainder() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_001,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::InsufficientAmount {
            found: 1_000_001,
            required: 1_213_000,
        })
    ));
}

#[test]
fn missing_input_for_alias_output() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Alias(alias_id, false))) if alias_id == alias_id_2
    ));
}

#[test]
fn missing_input_for_alias_output_2() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![
        Alias(2_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Alias(alias_id, false))) if alias_id == alias_id_2
    ));
}

#[test]
fn missing_input_for_alias_output_but_created() {
    let protocol_parameters = protocol_parameters();
    let alias_id_0 = AliasId::from_str(ALIAS_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_0,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(selected.is_ok());
}

#[test]
fn alias_in_output_and_sender() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Alias(1_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let alias_output = AliasOutputBuilder::from(inputs[0].output.as_alias())
        .with_state_index(inputs[0].output.as_alias().state_index() + 1)
        .finish_output(TOKEN_SUPPLY)
        .unwrap();
    let mut outputs = build_outputs(vec![Basic(
        1_000_000,
        BECH32_ADDRESS_ED25519_0,
        None,
        Some(BECH32_ADDRESS_ALIAS_1),
        None,
    )]);
    outputs.push(alias_output);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert!(unsorted_eq(&selected.outputs, &outputs));
}

#[test]
fn missing_ed25519_sender() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        Some(BECH32_ADDRESS_ED25519_1),
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_ed25519() && sender == Address::try_from_bech32(BECH32_ADDRESS_ED25519_1).unwrap().1
    ));
}

#[test]
fn missing_ed25519_issuer_created() {
    let protocol_parameters = protocol_parameters();
    let alias_id_0 = AliasId::from_str(ALIAS_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_0,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        Some(BECH32_ADDRESS_ED25519_1),
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Issuer(issuer))) if issuer.is_ed25519() && issuer == Address::try_from_bech32(BECH32_ADDRESS_ED25519_1).unwrap().1
    ));
}

#[test]
fn missing_ed25519_issuer_transition() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        Some(BECH32_ADDRESS_ED25519_1),
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(selected.is_ok());
}

#[test]
fn missing_alias_sender() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        Some(BECH32_ADDRESS_ALIAS_1),
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_alias() && sender == Address::try_from_bech32(BECH32_ADDRESS_ALIAS_1).unwrap().1
    ));
}

#[test]
fn missing_alias_issuer_created() {
    let protocol_parameters = protocol_parameters();
    let alias_id_0 = AliasId::from_str(ALIAS_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_0,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        Some(BECH32_ADDRESS_ALIAS_1),
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Issuer(issuer))) if issuer.is_alias() && issuer == Address::try_from_bech32(BECH32_ADDRESS_ALIAS_1).unwrap().1
    ));
}

#[test]
fn missing_alias_issuer_transition() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        Some(BECH32_ADDRESS_ALIAS_1),
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(selected.is_ok());
}

#[test]
fn missing_nft_sender() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        Some(BECH32_ADDRESS_NFT_1),
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_nft() && sender == Address::try_from_bech32(BECH32_ADDRESS_NFT_1).unwrap().1
    ));
}

#[test]
fn missing_nft_issuer_created() {
    let protocol_parameters = protocol_parameters();
    let alias_id_0 = AliasId::from_str(ALIAS_ID_0).unwrap();

    let inputs = build_inputs(vec![Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_0,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        Some(BECH32_ADDRESS_NFT_1),
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Issuer(issuer))) if issuer.is_nft() && issuer == Address::try_from_bech32(BECH32_ADDRESS_NFT_1).unwrap().1
    ));
}

#[test]
fn missing_nft_issuer_transition() {
    let protocol_parameters = protocol_parameters();
    let alias_id_2 = AliasId::from_str(ALIAS_ID_2).unwrap();

    let inputs = build_inputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_2,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        Some(BECH32_ADDRESS_NFT_1),
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(selected.is_ok());
}

#[test]
fn increase_alias_amount() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Alias(2_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let outputs = build_outputs(vec![Alias(
        3_000_000,
        alias_id_1,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert!(unsorted_eq(&selected.outputs, &outputs));
}

#[test]
fn decrease_alias_amount() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Alias(2_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let outputs = build_outputs(vec![Alias(
        1_000_000,
        alias_id_1,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert_eq!(selected.inputs.len(), 1);
    assert_eq!(selected.inputs[0], inputs[0]);
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(is_remainder_or_return(
                output,
                1_000_000,
                BECH32_ADDRESS_ED25519_0,
                None
            ));
        }
    });
}

#[test]
fn prefer_basic_to_alias() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Alias(1_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let outputs = build_outputs(vec![Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert_eq!(selected.inputs.len(), 1);
    assert_eq!(selected.inputs[0], inputs[1]);
    assert_eq!(selected.outputs, outputs);
}

#[test]
fn take_amount_from_alias_to_fund_basic() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Alias(2_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
        Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let outputs = build_outputs(vec![Basic(1_200_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(output.is_alias());
            assert_eq!(output.amount(), 1_800_000);
            assert_eq!(output.as_alias().native_tokens().len(), 0);
            assert_eq!(*output.as_alias().alias_id(), alias_id_1);
            assert_eq!(output.as_alias().unlock_conditions().len(), 2);
            assert_eq!(output.as_alias().features().len(), 0);
            assert_eq!(output.as_alias().immutable_features().len(), 0);
            assert_eq!(
                *output.as_alias().state_controller_address(),
                Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap().1
            );
            assert_eq!(
                *output.as_alias().governor_address(),
                Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap().1
            );
        }
    });
}

#[test]
fn alias_burn_should_not_validate_alias_sender() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Basic(2_000_000, BECH32_ADDRESS_ED25519_0, None, None, None),
        Alias(1_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let outputs = build_outputs(vec![Basic(
        2_000_000,
        BECH32_ADDRESS_ED25519_0,
        None,
        Some(BECH32_ADDRESS_ALIAS_1),
        None,
    )]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters)
        .burn(Burn::new().add_alias(alias_id_1))
        .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_alias() && sender == Address::try_from_bech32(BECH32_ADDRESS_ALIAS_1).unwrap().1
    ));
}

#[test]
fn alias_burn_should_not_validate_alias_address() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Basic(2_000_000, BECH32_ADDRESS_ALIAS_1, None, None, None),
        Alias(1_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let outputs = build_outputs(vec![Basic(2_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);

    let selected = InputSelection::new(inputs, outputs, protocol_parameters)
        .burn(Burn::new().add_alias(alias_id_1))
        .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Alias(alias_id, true))) if alias_id == alias_id_1
    ));
}

#[test]
fn alias_governance_transition_should_not_validate_alias_sender() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Basic(2_000_000, BECH32_ADDRESS_ED25519_0, None, None, None),
        Alias(1_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let mut outputs = build_outputs(vec![Basic(
        2_000_000,
        BECH32_ADDRESS_ED25519_0,
        None,
        Some(BECH32_ADDRESS_ALIAS_1),
        None,
    )]);
    outputs.push(inputs[1].output.clone());

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Sender(sender))) if sender.is_alias() && sender == Address::try_from_bech32(BECH32_ADDRESS_ALIAS_1).unwrap().1
    ));
}

#[test]
fn alias_governance_transition_should_not_validate_alias_address() {
    let protocol_parameters = protocol_parameters();
    let alias_id_1 = AliasId::from_str(ALIAS_ID_1).unwrap();

    let inputs = build_inputs(vec![
        Basic(2_000_000, BECH32_ADDRESS_ALIAS_1, None, None, None),
        Alias(1_000_000, alias_id_1, BECH32_ADDRESS_ED25519_0, None, None, None),
    ]);
    let mut outputs = build_outputs(vec![Basic(2_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);
    outputs.push(inputs[1].output.clone());

    let selected = InputSelection::new(inputs, outputs, protocol_parameters).select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Alias(alias_id, true))) if alias_id == alias_id_1
    ));
}

#[test]
fn transitioned_zero_alias_id_no_longer_is_zero() {
    let protocol_parameters = protocol_parameters();
    let alias_id_0 = AliasId::from_str(ALIAS_ID_0).unwrap();

    let inputs = build_inputs(vec![Alias(
        2_000_000,
        alias_id_0,
        BECH32_ADDRESS_ED25519_0,
        None,
        None,
        None,
    )]);
    let outputs = build_outputs(vec![Basic(1_000_000, BECH32_ADDRESS_ED25519_0, None, None, None)]);

    let selected = InputSelection::new(inputs.clone(), outputs.clone(), protocol_parameters)
        .select()
        .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(output.is_alias());
            assert_eq!(output.amount(), 1_000_000);
            assert_eq!(output.as_alias().native_tokens().len(), 0);
            assert_ne!(*output.as_alias().alias_id(), alias_id_0);
            assert_eq!(output.as_alias().unlock_conditions().len(), 2);
            assert_eq!(output.as_alias().features().len(), 0);
            assert_eq!(output.as_alias().immutable_features().len(), 0);
            assert_eq!(
                *output.as_alias().state_controller_address(),
                Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap().1
            );
            assert_eq!(
                *output.as_alias().governor_address(),
                Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap().1
            );
        }
    });
}
