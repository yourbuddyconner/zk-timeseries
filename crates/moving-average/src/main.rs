#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolValue;
use lib_timeseries::TimeSeries;

pub fn main() {
    // Read the timestamps and forecast values from the prover
    let timestamps = sp1_zkvm::io::read::<Vec<u64>>();
    let forecast_values = sp1_zkvm::io::read::<Vec<f64>>();
    let window_size = sp1_zkvm::io::read::<usize>();

    // Create a TimeSeries instance for statistical analysis
    let time_series = TimeSeries::new(timestamps, forecast_values);

    // Generate the public values struct for moving average
    let public_values = time_series.to_moving_average_public_values(window_size);

    // Encode the public values using ABI encoding
    let bytes = public_values.abi_encode();

    // Commit the encoded public values as output of the ZK proof
    sp1_zkvm::io::commit_slice(&bytes);
}
