//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use alloy_sol_types::SolType;
use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};
use tracing::log::{error, info};

/// The ELF file for the Succinct RISC-V zkVM moving average program.
pub const MOVING_AVERAGE_ELF: &[u8] =
    include_bytes!("../../../../elf/riscv32im-succinct-zkvm-moving-average-elf");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,

    #[clap(long, default_value = "3")]
    window_size: usize,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    // Generate some sample data
    let timestamps: Vec<u64> = (0..5).map(|i| i as u64 * 86400).collect();
    let forecast_values: Vec<f64> = (0..5).map(|i| i as f64 * 1.5).collect();

    stdin.write(&timestamps);
    stdin.write(&forecast_values);
    stdin.write(&args.window_size);

    info!("Timestamps: {:?}", timestamps);
    info!("Forecast values: {:?}", forecast_values);
    info!("Window size: {}", args.window_size);

    if args.execute {
        // Execute the program
        info!("Executing the program...");
        match client.execute(MOVING_AVERAGE_ELF, stdin).run() {
            Ok((output, report)) => {
                info!("Program executed successfully.");

                // Read the output.
                match lib_timeseries::MovingAveragePublicValuesStruct::abi_decode(
                    output.as_slice(),
                    true,
                ) {
                    Ok(decoded) => {
                        let lib_timeseries::MovingAveragePublicValuesStruct {
                            start_timestamp,
                            end_timestamp,
                            values_hash,
                            window_size,
                            moving_averages,
                        } = decoded;

                        info!("Decoded output:");
                        info!("Start timestamp: {}", start_timestamp);
                        info!("End timestamp: {}", end_timestamp);
                        info!("Values hash: {}", values_hash);
                        info!("Window size: {}", window_size);
                        info!("Moving averages: {:?}", moving_averages);
                    }
                    Err(e) => error!("Failed to decode output: {:?}", e),
                }

                // Record the number of cycles executed.
                info!("Number of cycles: {}", report.total_instruction_count());
            }
            Err(e) => error!("Execution failed: {:?}", e),
        }
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(MOVING_AVERAGE_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
