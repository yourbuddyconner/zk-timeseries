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
use timeseries_lib::PublicValuesStruct;
use tracing::log::{error, info};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const TIMESERIES_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,
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

    info!("Timestamps: {:?}", timestamps);
    info!("Forecast values: {:?}", forecast_values);

    if args.execute {
        // Execute the program
        info!("Executing the program...");
        match client.execute(TIMESERIES_ELF, stdin).run() {
            Ok((output, report)) => {
                info!("Program executed successfully.");

                // Read the output.
                match PublicValuesStruct::abi_decode(output.as_slice(), true) {
                    Ok(decoded) => {
                        let PublicValuesStruct {
                            timestamps,
                            forecast_values,
                            mean,
                            std_dev,
                        } = decoded;

                        info!("Decoded output:");
                        info!("Timestamps: {:?}", timestamps);
                        info!("Forecast values: {:?}", forecast_values);
                        info!("Mean: {}", mean);
                        info!("Standard Deviation: {}", std_dev);

                        // Record the number of cycles executed.
                        info!("Number of cycles: {}", report.total_instruction_count());
                    }
                    Err(e) => error!("Failed to decode output: {:?}", e),
                }
            }
            Err(e) => error!("Execution failed: {:?}", e),
        }
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(TIMESERIES_ELF);

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
