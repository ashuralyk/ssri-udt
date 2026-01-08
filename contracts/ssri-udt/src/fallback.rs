use alloc::{vec, vec::Vec};
use ckb_ssri_std::public_module_traits::udt::UDT;
use ckb_std::{ckb_constants::Source, debug, high_level::load_cell_lock_hash};
use core::cmp::Ordering;

use crate::{
    error::Error,
    modules::SSRIUDT,
    utils::{collect_inputs_amount, collect_outputs_amount},
};

pub fn fallback() -> Result<(), Error> {
    debug!("Entered fallback");
    let mut lock_hashes: Vec<[u8; 32]> = vec![];

    let mut index = 0;
    while let Ok(lock_hash) = load_cell_lock_hash(index, Source::Input) {
        lock_hashes.push(lock_hash);
        index += 1;
    }

    index = 0;
    while let Ok(lock_hash) = load_cell_lock_hash(index, Source::Output) {
        lock_hashes.push(lock_hash);
        index += 1;
    }

    let input_amount = collect_inputs_amount()?;
    let output_amount = collect_outputs_amount()?;

    match input_amount.cmp(&output_amount) {
        Ordering::Less => SSRIUDT::verify_mint(),
        Ordering::Equal => SSRIUDT::verify_transfer(),
        Ordering::Greater => return Err(Error::InsufficientBalance),
    }
}
