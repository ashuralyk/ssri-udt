use crate::{config::TYPE_ID_SCRIPT_CODE_HASH, error::Error};
use alloc::vec::Vec;
use ckb_ssri_std::public_module_traits::udt::UDT_LEN;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        core::ScriptHashType,
        packed::{CellOutput, Script},
        prelude::{Builder, Entity, Pack},
    },
    debug,
    high_level::{load_cell, load_cell_data, load_cell_lock_hash, load_cell_type, QueryIter},
};

pub fn collect_inputs_amount() -> Result<u128, Error> {
    debug!("Entered collect_inputs_amount");
    let mut buf = [0u8; UDT_LEN];

    let udt_list = QueryIter::new(load_cell_data, Source::GroupInput)
        .map(|data| {
            if data.len() == UDT_LEN {
                buf.copy_from_slice(&data);
                Ok(u128::from_le_bytes(buf))
            } else {
                Err(Error::Encoding)
            }
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(udt_list.into_iter().sum::<u128>())
}

pub fn collect_outputs_amount() -> Result<u128, Error> {
    debug!("Entered collect_outputs_amount");
    let mut buf = [0u8; UDT_LEN];

    let udt_list = QueryIter::new(load_cell_data, Source::GroupOutput)
        .map(|data| {
            if data.len() == UDT_LEN {
                buf.copy_from_slice(&data);
                // u128 is 16 bytes
                Ok(u128::from_le_bytes(buf))
            } else {
                Err(Error::Encoding)
            }
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(udt_list.into_iter().sum::<u128>())
}

pub fn check_owner_mode(owner_lockhash: &[u8; 32]) -> Result<bool, Error> {
    debug!("Entered check_owner_mode");
    let is_owner_mode = QueryIter::new(load_cell_lock_hash, Source::Input)
        .find(|lock_hash| owner_lockhash[..] == lock_hash[..])
        .is_some();
    debug!("Owner mode: {}", is_owner_mode);
    Ok(is_owner_mode)
}

pub fn find_ssri_config_cell(
    type_id_args: &[u8; 32],
    source: Source,
) -> Result<Option<(CellOutput, Vec<u8>)>, Error> {
    let ssri_type_script = Script::new_builder()
        .code_hash(TYPE_ID_SCRIPT_CODE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(type_id_args.to_vec().pack())
        .build();
    let ssri_config_cell =
        QueryIter::new(load_cell_type, source)
            .enumerate()
            .find_map(|(i, type_script)| match type_script {
                Some(type_script) => {
                    if type_script.as_slice() == ssri_type_script.as_slice() {
                        let cell = load_cell(i, source).unwrap();
                        let cell_data = load_cell_data(i, source).unwrap();
                        Some((cell, cell_data))
                    } else {
                        None
                    }
                }
                None => None,
            });
    Ok(ssri_config_cell)
}
