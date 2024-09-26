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
use ruint::Uint;
use timeseries_lib::{f64_to_u256, PublicValuesStruct, TimeSeries}; // Add this import

/// The main entry point for the SP1 program.
///
/// This function performs the following steps:
/// 1. Reads input data (timestamps and forecast values) from the prover.
/// 2. Creates a TimeSeries instance and calculates statistical measures.
/// 3. Converts the results to Solidity-compatible formats.
/// 4. Encodes the public values for verification in a smart contract.
/// 5. Commits the encoded data as public output of the ZK proof.
pub fn main() {
    // Read the number of data points from the prover
    let n = sp1_zkvm::io::read::<u32>();

    // Read the timestamps and forecast values from the prover
    let mut timestamps = Vec::with_capacity(n as usize);
    let mut forecast_values = Vec::with_capacity(n as usize);

    for _ in 0..n {
        timestamps.push(sp1_zkvm::io::read::<u64>());
        forecast_values.push(sp1_zkvm::io::read::<f64>());
    }

    // Create a TimeSeries instance for statistical analysis
    let time_series = TimeSeries::new(timestamps.clone(), forecast_values.clone());

    // Calculate mean and standard deviation of the forecast values
    let mean = time_series.mean();
    let std_dev = time_series.std_dev();

    // Convert f64 values to Uint<256, 4> for Solidity compatibility
    // This step is necessary because Solidity doesn't support floating-point numbers
    let forecast_values_uint: Vec<Uint<256, 4>> = forecast_values
        .iter()
        .map(|&v| Uint::from_str_radix(&f64_to_u256(v).to_string(), 10).unwrap())
        .collect();
    let mean_uint = Uint::from_str_radix(&f64_to_u256(mean).to_string(), 10).unwrap();
    let std_dev_uint = Uint::from_str_radix(&f64_to_u256(std_dev).to_string(), 10).unwrap();

    // Create a PublicValuesStruct with the calculated values
    // This struct mirrors a Solidity struct that will be used for verification
    let public_values = PublicValuesStruct {
        timestamps,
        forecast_values: forecast_values_uint,
        mean: mean_uint,
        std_dev: std_dev_uint,
    };

    // Encode the public values using ABI encoding
    // This creates a byte representation that can be decoded in Solidity
    let bytes = public_values.abi_encode();

    // Commit the encoded public values as output of the ZK proof
    // These values will be available for verification in the smart contract
    sp1_zkvm::io::commit_slice(&bytes);
}
