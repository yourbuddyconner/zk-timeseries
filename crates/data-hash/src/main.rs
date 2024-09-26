//! A SP1 program for time series analysis and forecasting.
//!
//! This program demonstrates how to perform time series calculations within a zero-knowledge proof system.
//! It takes a series of timestamps and corresponding forecast values as input, performs statistical
//! calculations, and outputs the results in a format compatible with Solidity smart contracts.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolValue;
use lib_timeseries::TimeSeries;

/// The main entry point for the SP1 program.
///
/// This function performs the following steps:
/// 1. Reads input data (timestamps and forecast values) from the prover.
/// 2. Creates a TimeSeries instance and calculates statistical measures.
/// 3. Converts the results to Solidity-compatible formats.
/// 4. Encodes the public values for verification in a smart contract.
/// 5. Commits the encoded data as public output of the ZK proof.
pub fn main() {
    // Read the timestamps and forecast values from the prover
    let timestamps = sp1_zkvm::io::read::<Vec<u64>>();
    let forecast_values = sp1_zkvm::io::read::<Vec<f64>>();

    // Create a TimeSeries instance for statistical analysis
    let time_series = TimeSeries::new(timestamps, forecast_values);

    // Generate the public values struct
    let public_values = time_series.to_public_values();

    // Encode the public values using ABI encoding
    let bytes = public_values.abi_encode();

    // Commit the encoded public values as output of the ZK proof
    sp1_zkvm::io::commit_slice(&bytes);
}
