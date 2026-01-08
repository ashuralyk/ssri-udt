#![allow(unexpected_cfgs)]
#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(test)]
extern crate alloc;

use alloc::{borrow::Cow, vec};
use ckb_ssri_std::{public_module_traits::udt::UDT, utils::should_fallback};
use ckb_ssri_std_proc_macro::ssri_methods;
use ckb_std::{
    ckb_types::{
        packed::{Byte32, Bytes, Script, ScriptBuilder, Transaction},
        prelude::*,
    },
    debug,
    syscalls::{pipe, write},
};
use serde_molecule::from_slice;

#[cfg(not(test))]
use ckb_std::default_alloc;

#[cfg(not(test))]
ckb_std::entry!(program_entry);

#[cfg(not(test))]
default_alloc!();

mod config;
mod error;
mod fallback;
mod modules;
mod molecule;
mod utils;

use error::Error;

fn program_entry_wrap() -> Result<(), Error> {
    let argv = ckb_std::env::argv();

    if should_fallback()? {
        return Ok(fallback::fallback()?);
    }

    debug!("Entering ssri_methods");
    // NOTE: The following part is an entry function acting as an controller for all SSRI methods and also handles the deserialization/serialization.
    // In the future, methods can be reflected automatically from traits using procedural macros and entry methods to other methods of the same trait for a more concise and maintainable entry function.
    let res: Cow<'static, [u8]> = ssri_methods!(
        argv: &argv,
        invalid_method: Error::SSRIMethodsNotFound,
        invalid_args: Error::SSRIMethodsArgsInvalid,
        "UDT.name" => Ok(Cow::from(modules::SSRIUDT::name()?.to_vec())),
        "UDT.symbol" => Ok(Cow::from(modules::SSRIUDT::symbol()?.to_vec())),
        "UDT.decimals" => Ok(Cow::from(modules::SSRIUDT::decimals()?.to_le_bytes().to_vec())),
        "UDT.icon" => Ok(Cow::from(modules::SSRIUDT::icon()?.to_vec())),
        "UDT.transfer" => {
            debug!("program_entry_wrap | Entered UDT.transfer");
            let to_lock_vec_molecule = molecule::ScriptVec::from_slice(decode_hex(argv[2].as_ref())?.as_slice()).map_err(|_|Error::MoleculeVerificationError)?;
            let mut to_lock_vec: Vec<Script> = vec![];
            for script in to_lock_vec_molecule.into_iter() {
                let parsed_script = ScriptBuilder::default()
                    .code_hash(Byte32::from_slice(script.as_reader().code_hash().to_entity().as_slice()).map_err(|_|Error::MoleculeVerificationError)?)
                    .hash_type(script.as_reader().hash_type().to_entity())
                    .args(Bytes::from_slice(script.as_reader().args().to_entity().as_slice()).map_err(|_|Error::MoleculeVerificationError)?)
                    .build();
                to_lock_vec.push(parsed_script);
            }

            let to_amount_bytes = decode_hex(argv[3].as_ref())?;
            let to_amount_vec: Vec<u128> = to_amount_bytes[4..]
                .chunks(16)
                .map(|chunk| {
                    return u128::from_le_bytes(chunk.try_into().unwrap())}
                )
                .collect();

            if argv[2].is_empty() || argv[3].is_empty() || to_lock_vec.len() != to_amount_vec.len() {
                Err(Error::SSRIMethodsArgsInvalid)?;
            }

            let tx: Option<Transaction> = if argv[1].as_ref().is_empty() {
                None
            } else {
                Some(Transaction::from_compatible_slice(&decode_hex(argv[1].as_ref())?).map_err(|_|Error::MoleculeVerificationError)?)
            };

            Ok(Cow::from(modules::SSRIUDT::transfer(tx, to_lock_vec, to_amount_vec)?.as_bytes().to_vec()))
        },
        "UDT.mint" => {
            debug!("program_entry_wrap | Entered UDT.mint");
            let to_lock_vec_molecule = molecule::ScriptVec::from_slice(decode_hex(argv[2].as_ref())?.as_slice()).map_err(|_|Error::MoleculeVerificationError)?;
            let mut to_lock_vec: Vec<Script> = vec![];
            for script in to_lock_vec_molecule.into_iter() {
                let parsed_script = ScriptBuilder::default()
                    .code_hash(Byte32::from_slice(script.as_reader().code_hash().to_entity().as_slice()).map_err(|_|Error::MoleculeVerificationError)?)
                    .hash_type(script.as_reader().hash_type().to_entity())
                    .args(Bytes::from_slice(script.as_reader().args().to_entity().as_slice()).map_err(|_|Error::MoleculeVerificationError)?)
                    .build();
                to_lock_vec.push(parsed_script);
            }
            debug!("program_entry_wrap | to_lock_vec: {:?}", to_lock_vec);

            let to_amount_bytes = decode_hex(argv[3].as_ref())?;
            let to_amount_vec: Vec<u128> = to_amount_bytes[4..]
                .chunks(16)
                .map(|chunk| {
                    return u128::from_le_bytes(chunk.try_into().unwrap())}
                )
                .collect();
            debug!("program_entry_wrap | to_amount_vec: {:?}", to_amount_vec);

            if argv[2].is_empty() || argv[3].is_empty() || to_lock_vec.len() != to_amount_vec.len() {
                Err(Error::SSRIMethodsArgsInvalid)?;
            }

            let tx: Option<Transaction> = if argv[1].as_ref().is_empty() {
                None
            } else {
                Some(Transaction::from_compatible_slice(&decode_hex(argv[1].as_ref())?).map_err(|_|Error::MoleculeVerificationError)?)
            };

            Ok(Cow::from(modules::SSRIUDT::mint(tx, to_lock_vec, to_amount_vec)?.as_bytes().to_vec()))
        },
        "SSRIUDT.create" => {
            debug!("program_entry_wrap | Entered SSRIUDT.create");
            let tx_bytes = decode_hex(argv.get(1).ok_or(Error::SSRIMethodsArgsInvalid)?)?;
            let owner_lock_bytes = decode_hex(argv.get(2).ok_or(Error::SSRIMethodsArgsInvalid)?)?;
            let ssri_metadata_bytes = decode_hex(argv.get(3).ok_or(Error::SSRIMethodsArgsInvalid)?)?;

            let tx = Transaction::from_compatible_slice(&tx_bytes).map_err(|_|Error::MoleculeVerificationError)?;
            let owner_lock = Script::from_compatible_slice(&owner_lock_bytes).map_err(|_|Error::MoleculeVerificationError)?;
            let ssri_metadata: modules::SSRIMetadata = from_slice(&ssri_metadata_bytes, false).map_err(|_|Error::MoleculeVerificationError)?;
            Ok(Cow::from(ssri_metadata.generate_ssri_create_tx(tx, owner_lock)?.as_bytes().to_vec()))
        },
    )?;
    let pipe = pipe()?;
    write(pipe.1, &res)?;
    Ok(())
}

pub fn program_entry() -> i8 {
    match program_entry_wrap() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}
