use ckb_std::ckb_types::prelude::Entity;
use ckb_std::ckb_types::{bytes::Bytes, packed::*, prelude::*};
use ckb_testtool::ckb_types::core::TransactionBuilder;

use crate::utils::build_test_context;

#[test]
pub fn test_transfer() {
    let mut test_context = build_test_context();

    let wallet_amount: Uint128 = 20000000000u128.pack();
    let transfer_amount: Uint128 = 10000000000u128.pack();
    let change_amount: Uint128 = (20000000000u128 - 10000000000u128).pack();

    let normal_udt_input_outpoint = test_context.context.create_cell(
        CellOutput::new_builder()
            .capacity(100u64.pack())
            .lock(test_context.normal_user_a_lock_script.clone())
            .type_(Some(test_context.ssri_udt_type_script.clone()).pack())
            .build(),
        wallet_amount.as_bytes(),
    );

    let normal_inputs = vec![CellInput::new_builder()
        .previous_output(normal_udt_input_outpoint.clone())
        .build()];

    let normal_udt_output = CellOutput::new_builder()
        .capacity(100u64.pack())
        .lock(test_context.normal_user_b_lock_script.clone())
        .type_(Some(test_context.ssri_udt_type_script.clone()).pack())
        .build();

    let normal_udt_change_output = CellOutput::new_builder()
        .capacity(100u64.pack())
        .lock(test_context.normal_user_a_lock_script.clone())
        .type_(Some(test_context.ssri_udt_type_script.clone()).pack())
        .build();

    let normal_outputs = vec![normal_udt_output.clone(), normal_udt_change_output.clone()];
    let outputs_data = vec![transfer_amount.raw_data(), change_amount.raw_data()];

    let normal_transfer_tx = TransactionBuilder::default()
        .inputs(normal_inputs.clone())
        .outputs(normal_outputs.clone())
        .outputs_data(outputs_data.clone().pack())
        .cell_deps(vec![
            test_context.ssri_udt_dep.clone(),
            test_context.always_success_dep.clone(),
        ])
        .build();

    let normal_transfer_tx = normal_transfer_tx.as_advanced_builder().build();

    let normal_cycles = test_context
        .context
        .verify_tx(&normal_transfer_tx, u64::MAX)
        .expect("Normal Tx Failed");
    println!("Normal Tx cycles: {}", normal_cycles);
}

#[test]
pub fn test_mint() {
    println!("Entered test_mint");
    let mut test_context = build_test_context();

    let mint_amount: Uint128 = 20000000000u128.pack();

    let admin_out_point = test_context.context.create_cell(
        CellOutput::new_builder()
            .capacity(10000u64.pack())
            .lock(test_context.admin_lock_script.clone())
            .build(),
        Bytes::default(),
    );

    let admin_inputs = vec![CellInput::new_builder()
        .previous_output(admin_out_point.clone())
        .build()];

    let normal_udt_output = CellOutput::new_builder()
        .capacity(100u64.pack())
        .lock(test_context.normal_user_b_lock_script.clone())
        .type_(Some(test_context.ssri_udt_type_script.clone()).pack())
        .build();

    let outputs_data = vec![mint_amount.raw_data()];

    let normal_mint_tx = TransactionBuilder::default()
        .inputs(admin_inputs.clone())
        .output(normal_udt_output.clone())
        .outputs_data(outputs_data.clone().pack())
        .cell_deps(vec![
            test_context.ssri_udt_dep.clone(),
            test_context.always_success_dep.clone(),
            test_context.ssri_metadata_dep.clone(),
        ])
        .build();

    let normal_mint_tx = normal_mint_tx.as_advanced_builder().build();

    let normal_cycles = test_context
        .context
        .verify_tx(&normal_mint_tx, u64::MAX)
        .expect("Normal Mint Tx Failed");
    println!("Normal Mint Tx cycles: {}", normal_cycles);

    let user_a_out_point = test_context.context.create_cell(
        CellOutput::new_builder()
            .capacity(100u64.pack())
            .lock(test_context.normal_user_a_lock_script.clone())
            .build(),
        mint_amount.as_bytes(),
    );

    let user_a_inputs = vec![CellInput::new_builder()
        .previous_output(user_a_out_point.clone())
        .build()];
    let unauthorized_mint_tx = TransactionBuilder::default()
        .inputs(user_a_inputs.clone())
        .output(normal_udt_output.clone())
        .outputs_data(outputs_data.clone().pack())
        .cell_deps(vec![
            test_context.ssri_udt_dep.clone(),
            test_context.always_success_dep.clone(),
            test_context.ssri_metadata_dep.clone(),
        ])
        .build();

    let unauthorized_mint_tx = unauthorized_mint_tx.as_advanced_builder().build();

    let unauthorized_mint_err = test_context
        .context
        .verify_tx(&unauthorized_mint_tx, u64::MAX)
        .unwrap_err();

    println!(
        "Expected Unauthorized Mint Tx Error: {:?}",
        unauthorized_mint_err
    );
}
