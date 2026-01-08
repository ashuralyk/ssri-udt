use ckb_hash::blake2b_256;
use ckb_std::{
    ckb_types::{bytes::Bytes, core::ScriptHashType, packed::*, prelude::*},
    high_level::encode_hex,
};
use ckb_testtool::{
    builtin::ALWAYS_SUCCESS, ckb_chain_spec::consensus::TYPE_ID_CODE_HASH, context::Context,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_molecule::to_vec;

use crate::Loader;

pub fn method_path(name: impl AsRef<[u8]>) -> u64 {
    u64::from_le_bytes(blake2b_256(name)[0..8].try_into().unwrap())
}

pub fn method_path_hex(name: impl AsRef<[u8]>) -> String {
    let method_path = method_path(name);
    let method_path_in_bytes = method_path.to_le_bytes();
    let method_path_hex = format!(
        "0x{:?}",
        encode_hex(&method_path_in_bytes).into_string().unwrap()
    )
    .replace("\"", "");
    method_path_hex
}

pub async fn get_ssri_response(payload: serde_json::Value) -> serde_json::Value {
    let url = "http://localhost:9090";

    let client = Client::new();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    // Assert that the request was successful (status 200)
    assert!(
        response.status().is_success(),
        "Response was not successful"
    );

    let response_json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    response_json
}

#[derive(Serialize, Deserialize)]
pub struct SSRIMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub icon: String,
}

pub struct PausableUDTTestContext {
    pub context: Context,
    pub always_success_dep: CellDep,
    pub ssri_udt_dep: CellDep,
    pub ssri_udt_type_script: Script,
    pub ssri_metadata_dep: CellDep,
    pub admin_lock_script: Script,
    pub normal_user_a_lock_script: Script,
    pub normal_user_b_lock_script: Script,
}

pub fn build_test_context() -> PausableUDTTestContext {
    let admin_args = String::from("00018fd14cc327648651dc0ac81ec6dd63a9ab376e61");
    let normal_user_a_args = String::from("00018fd14cc327648651dc0ac81ec6dd63a9ab376e62");
    let normal_user_b_args = String::from("00018fd14cc327648651dc0ac81ec6dd63a9ab376e63");
    let paused_user_args = String::from("00018fd14cc327648651dc0ac81ec6dd63a9ab376e64");

    let mut context = Context::default();
    let loader = Loader::default();
    let ssri_udt_bin = loader.load_binary("ssri-udt");
    let ssri_udt_out_point = context.deploy_cell(ssri_udt_bin);
    let ssri_udt_dep = CellDep::new_builder()
        .out_point(ssri_udt_out_point.clone())
        .build();
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let always_success_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();
    let _always_success_script = context
        .build_script(&always_success_out_point, Bytes::default())
        .expect("script");

    let admin_lock_script = context
        .build_script(&&always_success_out_point.clone(), Bytes::from(admin_args))
        .expect("script");
    let normal_user_a_lock_script = context
        .build_script(
            &always_success_out_point.clone(),
            Bytes::from(normal_user_a_args),
        )
        .expect("script");
    let normal_user_b_lock_script = context
        .build_script(
            &always_success_out_point.clone(),
            Bytes::from(normal_user_b_args),
        )
        .expect("script");
    let paused_user_lock_script = context
        .build_script(
            &always_success_out_point.clone(),
            Bytes::from(paused_user_args),
        )
        .expect("script");

    let paused_user_lock_script_hash_byte32 = paused_user_lock_script.calc_script_hash();
    let paused_user_lock_script_hash_hex =
        encode_hex(paused_user_lock_script_hash_byte32.as_slice());

    println!(
        "paused_user_lock_script_hash_hex: {}",
        paused_user_lock_script_hash_hex.into_string().unwrap()
    );

    let ssri_metadata = SSRIMetadata {
        name: String::from("Test UDT"),
        symbol: String::from("TEST"),
        decimals: 8,
        icon: String::from("https://example.com/icon.png"),
    };
    let ssri_metadata_args = vec![0; 32];
    let ssri_metadata_type_script = Script::new_builder()
        .code_hash(TYPE_ID_CODE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(ssri_metadata_args.pack())
        .build();
    println!(
        "ssri_metadata_type_script len: {}",
        ssri_metadata_type_script.as_slice().len()
    );
    let ssri_metadat_cell = CellOutput::new_builder()
        .lock(admin_lock_script.clone())
        .type_(Some(ssri_metadata_type_script.clone()).pack())
        .build();
    let ssri_metadata_out_point = context.create_cell(
        ssri_metadat_cell,
        to_vec(&ssri_metadata, false).unwrap().into(),
    );
    let ssri_metadata_dep = CellDep::new_builder()
        .out_point(ssri_metadata_out_point.clone())
        .build();

    let ssri_udt_type_script = context
        .build_script(
            &ssri_udt_out_point,
            // admin_lock_script.calc_script_hash().as_bytes(),
            ssri_metadata_args.into(),
        )
        .expect("script");

    PausableUDTTestContext {
        context,
        always_success_dep,
        ssri_udt_dep,
        ssri_udt_type_script,
        ssri_metadata_dep,
        admin_lock_script,
        normal_user_a_lock_script,
        normal_user_b_lock_script,
    }
}
