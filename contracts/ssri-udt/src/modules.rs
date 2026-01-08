use alloc::{string::String, vec::Vec};
use ckb_hash::new_blake2b;
use ckb_ssri_std::{
    public_module_traits::udt::UDT,
    utils::high_level::{find_cell_data_by_out_point, find_out_point_by_type},
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        bytes::Bytes,
        core::ScriptHashType,
        packed::{
            Byte32Vec, BytesVec, BytesVecBuilder, CellDep, CellDepVecBuilder, CellInputVec,
            CellOutputBuilder, CellOutputVecBuilder, OutPoint, RawTransactionBuilder, Script,
            ScriptBuilder, ScriptOptBuilder, Transaction, TransactionBuilder, Uint32, Uint64,
        },
        prelude::*,
    },
    debug,
    high_level::load_script,
};
use serde::{Deserialize, Serialize};
use serde_molecule::{from_slice, to_vec};

use crate::{
    config::TYPE_ID_SCRIPT_CODE_HASH,
    error::Error,
    utils::{
        check_owner_mode, collect_inputs_amount, collect_outputs_amount, find_ssri_config_cell,
    },
};

#[derive(Serialize, Deserialize)]
pub struct SSRIMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub icon: String,
}

impl SSRIMetadata {
    pub fn new_from_onchain_search() -> Result<Self, Error> {
        let ssri_config_outpoint = Self::search_outpoint()?;
        let ssri_config = find_cell_data_by_out_point(ssri_config_outpoint)?;
        Ok(from_slice(&ssri_config, false)?)
    }

    // must run at `script` level
    pub fn search_outpoint() -> Result<OutPoint, Error> {
        let type_id_args: Vec<u8> = load_script()?.args().unpack();
        let ssri_config_type_script = Script::new_builder()
            .code_hash(TYPE_ID_SCRIPT_CODE_HASH.pack())
            .hash_type(ScriptHashType::Type.into())
            .args(type_id_args.pack())
            .build();
        let ssri_config_outpoint = find_out_point_by_type(ssri_config_type_script)?;
        Ok(ssri_config_outpoint)
    }

    pub fn generate_ssri_create_tx(
        &self,
        tx: Transaction,
        owner_lock: Script,
    ) -> Result<Transaction, Error> {
        let Some(first_input_cell_input) = tx.raw().inputs().get(0) else {
            return Err(Error::InvalidTransactionInputs);
        };
        let type_id_args: [u8; 32] = {
            let mut hasher = new_blake2b();
            hasher.update(first_input_cell_input.as_slice());
            hasher.update(&(tx.raw().outputs().len() as u32).to_le_bytes());
            let mut ret = [0; 32];
            hasher.finalize(&mut ret);
            ret
        };
        let tx_builder = tx.raw().as_builder();
        let mut outputs_vec_builder = tx.raw().outputs().as_builder();
        let mut outputs_data_vec_builder = tx.raw().outputs_data().as_builder();
        outputs_vec_builder = outputs_vec_builder.push(
            // type_id script
            CellOutputBuilder::default()
                .lock(owner_lock)
                .type_(
                    Some(
                        ScriptBuilder::default()
                            .code_hash(TYPE_ID_SCRIPT_CODE_HASH.pack())
                            .hash_type(ScriptHashType::Type.into())
                            .args(type_id_args.to_vec().pack())
                            .build(),
                    )
                    .pack(),
                )
                .build(),
        );
        outputs_data_vec_builder = outputs_data_vec_builder.push(to_vec(&self, false)?.pack());
        Ok(tx
            .as_builder()
            .raw(
                tx_builder
                    .outputs(outputs_vec_builder.build())
                    .outputs_data(outputs_data_vec_builder.build())
                    .build(),
            )
            .build())
    }
}

pub struct SSRIUDT;

// #[ssri_module]
impl UDT for SSRIUDT {
    type Error = Error;

    // #[ssri_method(level = "script", transaction = true)]
    fn transfer(
        tx: Option<Transaction>,
        to_lock_vec: Vec<Script>,
        to_amount_vec: Vec<u128>,
    ) -> Result<Transaction, Error> {
        debug!("Entered UDT::transfer");
        if to_amount_vec.len() != to_lock_vec.len() {
            return Err(Error::SSRIMethodsArgsInvalid);
        }
        let tx_builder = match tx {
            Some(ref tx) => tx.clone().as_builder(),
            None => TransactionBuilder::default(),
        };
        let raw_tx_builder = match tx {
            Some(ref tx) => tx.clone().raw().as_builder(),
            None => RawTransactionBuilder::default(),
        };

        let mut cell_output_vec_builder = match tx {
            Some(ref tx) => tx.clone().raw().outputs().as_builder(),
            None => CellOutputVecBuilder::default(),
        };

        for to_lock in to_lock_vec.iter() {
            let new_transfer_output = CellOutputBuilder::default()
                .type_(
                    ScriptOptBuilder::default()
                        .set(Some(load_script()?))
                        .build(),
                )
                .capacity(Uint64::default())
                .lock(to_lock.clone())
                .build();
            cell_output_vec_builder = cell_output_vec_builder.push(new_transfer_output);
        }

        let mut outputs_data_builder = match tx {
            Some(ref tx) => tx.clone().raw().outputs_data().as_builder(),
            None => BytesVecBuilder::default(),
        };

        for to_amount in to_amount_vec.iter() {
            outputs_data_builder = outputs_data_builder.push(to_amount.pack().as_bytes().pack());
        }

        Ok(tx_builder
            .raw(
                raw_tx_builder
                    .version(
                        tx.clone()
                            .map(|t| t.raw().version())
                            .unwrap_or_else(|| Uint32::default()),
                    )
                    .header_deps(
                        tx.clone()
                            .map(|t| t.raw().header_deps())
                            .unwrap_or_else(|| Byte32Vec::default()),
                    )
                    .inputs(
                        tx.clone()
                            .map(|t| t.raw().inputs())
                            .unwrap_or_else(|| CellInputVec::default()),
                    )
                    .outputs(cell_output_vec_builder.build())
                    .outputs_data(outputs_data_builder.build())
                    .build(),
            )
            .witnesses(
                tx.clone()
                    .map(|t| t.witnesses())
                    .unwrap_or_else(|| BytesVec::default()),
            )
            .build())
    }

    fn verify_transfer() -> Result<(), Self::Error> {
        debug!("Entered UDT::verify_transfer");
        let inputs_amount = collect_inputs_amount()?;
        let outputs_amount = collect_outputs_amount()?;

        if inputs_amount < outputs_amount {
            return Err(Error::InsufficientBalance);
        }
        debug!("inputs_amount: {}", inputs_amount);
        debug!("outputs_amount: {}", outputs_amount);
        Ok(())
    }

    // #[ssri_method(level = "script")]
    fn name() -> Result<Bytes, Self::Error> {
        let ssri_metadata =
            SSRIMetadata::new_from_onchain_search().map_err(|_| Error::SSRIConfigNotFound)?;
        Ok(ssri_metadata.name.into_bytes().into())
    }

    // #[ssri_method(level = "script")]
    fn symbol() -> Result<Bytes, Self::Error> {
        let ssri_metadata =
            SSRIMetadata::new_from_onchain_search().map_err(|_| Error::SSRIConfigNotFound)?;
        Ok(ssri_metadata.symbol.into_bytes().into())
    }

    // #[ssri_method(level = "script")]
    fn decimals() -> Result<u8, Self::Error> {
        let ssri_metadata =
            SSRIMetadata::new_from_onchain_search().map_err(|_| Error::SSRIConfigNotFound)?;
        Ok(ssri_metadata.decimals)
    }

    // #[ssri_method(level = "script")]
    fn icon() -> Result<Bytes, Self::Error> {
        let ssri_metadata =
            SSRIMetadata::new_from_onchain_search().map_err(|_| Error::SSRIConfigNotFound)?;
        Ok(ssri_metadata.icon.into_bytes().into())
    }

    // #[ssri_method(level = "script", transaction = true)]
    fn mint(
        tx: Option<Transaction>,
        to_lock_vec: Vec<Script>,
        to_amount_vec: Vec<u128>,
    ) -> Result<Transaction, Error> {
        debug!("Entered UDT::mint");
        let tx_builder = match tx {
            Some(ref tx) => tx.clone().as_builder(),
            None => TransactionBuilder::default(),
        };
        let raw_tx_builder = match tx {
            Some(ref tx) => tx.clone().raw().as_builder(),
            None => RawTransactionBuilder::default(),
        };

        let mut cell_output_vec_builder = match tx {
            Some(ref tx) => tx.clone().raw().outputs().as_builder(),
            None => CellOutputVecBuilder::default(),
        };

        for to_lock in to_lock_vec.iter() {
            let new_mint_output = CellOutputBuilder::default()
                .type_(
                    ScriptOptBuilder::default()
                        .set(Some(load_script()?))
                        .build(),
                )
                .lock(to_lock.clone())
                .build();
            cell_output_vec_builder = cell_output_vec_builder.push(new_mint_output);
        }

        let mut outputs_data_builder = match tx {
            Some(ref tx) => tx.clone().raw().outputs_data().as_builder(),
            None => BytesVecBuilder::default(),
        };

        for to_amount in to_amount_vec.iter() {
            outputs_data_builder = outputs_data_builder.push(to_amount.pack().as_bytes().pack());
        }

        let mut cell_dep_vec_builder: CellDepVecBuilder = match tx {
            Some(ref tx) => tx.clone().raw().cell_deps().as_builder(),
            None => CellDepVecBuilder::default(),
        };

        let ssri_metadata_outpoint = SSRIMetadata::search_outpoint()?;
        let ssri_metadata_celldep = CellDep::new_builder()
            .out_point(ssri_metadata_outpoint)
            .build();
        cell_dep_vec_builder = cell_dep_vec_builder.push(ssri_metadata_celldep);

        Ok(tx_builder
            .raw(
                raw_tx_builder
                    .version(
                        tx.clone()
                            .map(|t| t.raw().version())
                            .unwrap_or_else(|| Uint32::default()),
                    )
                    .cell_deps(cell_dep_vec_builder.build())
                    .header_deps(
                        tx.clone()
                            .map(|t| t.raw().header_deps())
                            .unwrap_or_else(|| Byte32Vec::default()),
                    )
                    .inputs(
                        tx.clone()
                            .map(|t| t.raw().inputs())
                            .unwrap_or_else(|| CellInputVec::default()),
                    )
                    .outputs(cell_output_vec_builder.build())
                    .outputs_data(outputs_data_builder.build())
                    .build(),
            )
            .witnesses(
                tx.clone()
                    .map(|t| t.witnesses())
                    .unwrap_or_else(|| BytesVec::default()),
            )
            .build())
    }

    fn verify_mint() -> Result<(), Self::Error> {
        debug!("Entered UDT::verify_mint");
        let script = load_script()?;
        let ssri_config_typeid_args: [u8; 32] = script
            .args()
            .raw_data()
            .to_vec()
            .try_into()
            .map_err(|_| Error::InvalidUDTArgs)?;
        let Some((ssri_config_cell, config_data)) =
            find_ssri_config_cell(&ssri_config_typeid_args, Source::CellDep).or(
                find_ssri_config_cell(&ssri_config_typeid_args, Source::Output),
            )?
        else {
            return Err(Error::SSRIConfigNotFound);
        };
        from_slice::<SSRIMetadata>(&config_data, false)
            .map_err(|_| Error::SSRIConfigInvalidDataFormat)?;
        let owner_lockhash = ssri_config_cell.lock().calc_script_hash().unpack();
        if check_owner_mode(&owner_lockhash)? {
            return Ok(());
        } else {
            return Err(Error::NoMintPermission);
        }
    }
}
