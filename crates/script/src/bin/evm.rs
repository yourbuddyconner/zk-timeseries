//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! EVM-Compatible proof generated which can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system groth16
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system plonk
//! ```

use alloy_sol_types::SolType;
use clap::{Parser, ValueEnum};
use lib_timeseries::{MovingAveragePublicValuesStruct, PublicValuesStruct};
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use std::path::PathBuf;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const TIMESERIES_ELF: &[u8] =
    include_bytes!("../../../../elf/riscv32im-succinct-zkvm-data-hash-elf");
pub const MOVING_AVERAGE_ELF: &[u8] =
    include_bytes!("../../../../elf/riscv32im-succinct-zkvm-moving-average-elf");

/// The arguments for the EVM command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct EVMArgs {
    #[clap(long, default_value = "5")]
    n: u32,
    #[clap(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
    #[clap(long)]
    moving_average: bool,
    #[clap(long, default_value = "3")]
    window_size: usize,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1TimeSeriesProofFixture {
    start_timestamp: String,
    end_timestamp: String,
    values_hash: String,
    window_size: Option<String>,
    moving_averages: Option<Vec<String>>,
    mean: Option<String>,
    median: Option<String>,
    std_dev: Option<String>,
    vkey: String,
    public_values: String,
    proof: String,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = EVMArgs::parse();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let elf = if args.moving_average {
        MOVING_AVERAGE_ELF
    } else {
        TIMESERIES_ELF
    };
    let (pk, vk) = client.setup(elf);

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    // Generate some sample data
    let timestamps: Vec<u64> = (0..args.n).map(|i| i as u64 * 86400).collect();
    let forecast_values: Vec<f64> = (0..args.n).map(|i| i as f64 * 1.5).collect();

    // Write the sample data to stdin
    stdin.write(&timestamps);
    stdin.write(&forecast_values);
    if args.moving_average {
        stdin.write(&args.window_size);
    }

    println!("n: {}", args.n);
    println!("Proof System: {:?}", args.system);

    // Generate the proof based on the selected proof system.
    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, stdin).groth16().run(),
    }
    .expect("failed to generate proof");

    create_proof_fixture(&proof, &vk, args.system, args.moving_average);
}

/// Create a fixture for the given proof.
fn create_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: ProofSystem,
    is_moving_average: bool,
) {
    let bytes = proof.public_values.as_slice();
    let fixture = if is_moving_average {
        let MovingAveragePublicValuesStruct {
            start_timestamp,
            end_timestamp,
            values_hash,
            window_size,
            moving_averages,
        } = MovingAveragePublicValuesStruct::abi_decode(bytes, false).unwrap();

        SP1TimeSeriesProofFixture {
            start_timestamp: start_timestamp.to_string(),
            end_timestamp: end_timestamp.to_string(),
            values_hash: values_hash.to_string(),
            window_size: Some(window_size.to_string()),
            moving_averages: Some(moving_averages.iter().map(|v| v.to_string()).collect()),
            mean: None,
            median: None,
            std_dev: None,
            vkey: vk.bytes32().to_string(),
            public_values: format!("0x{}", hex::encode(bytes)),
            proof: format!("0x{}", hex::encode(proof.bytes())),
        }
    } else {
        // Deserialize the public values.
        let PublicValuesStruct {
            start_timestamp,
            end_timestamp,
            values_hash,
            mean,
            median,
            std_dev,
        } = PublicValuesStruct::abi_decode(bytes, false).unwrap();

        // Create the testing fixture so we can test things end-to-end.
        SP1TimeSeriesProofFixture {
            start_timestamp: start_timestamp.to_string(),
            end_timestamp: end_timestamp.to_string(),
            values_hash: values_hash.to_string(),
            mean: Some(mean.to_string()),
            median: Some(median.to_string()),
            std_dev: Some(std_dev.to_string()),
            window_size: None,
            moving_averages: None,
            vkey: vk.bytes32().to_string(),
            public_values: format!("0x{}", hex::encode(bytes)),
            proof: format!("0x{}", hex::encode(proof.bytes())),
        }
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values which are publicly committed to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
