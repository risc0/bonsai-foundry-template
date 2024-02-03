// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::Write;

use alloy_primitives::FixedBytes;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use risc0_build::GuestListEntry;
use risc0_zkvm::compute_image_id;

use crate::{
    eth::{self},
    prover,
    snark::Proof,
};

/// CLI commands.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Command {
    /// Runs the RISC-V ELF binary.
    Query {
        /// The name of the guest binary
        guest_binary: String,

        /// The input to provide to the guest binary
        input: Option<String>,
    },
    /// Runs the RISC-V ELF binary on Bonsai
    /// and publish the result to Ethererum.
    Publish {
        /// Ethereum chain ID
        #[clap(long)]
        chain_id: u64,

        /// Ethereum Node endpoint.
        #[clap(long, env)]
        eth_wallet_private_key: String,

        /// Ethereum Node endpoint.
        #[clap(long)]
        rpc_url: String,

        /// Application's contract address on Ethereum
        #[clap(long)]
        contract: String,

        /// The name of the guest binary
        #[clap(long)]
        guest_binary: String,

        /// The input to provide to the guest binary
        #[clap(short, long)]
        input: String,
    },
}

/// GuestInterface for parsing guest input and encoding calldata.
pub trait GuestInterface {
    /// Input type expected by the guest from `env::read()`.
    type Input: serde::Serialize;

    /// Journal data type written by the guest via `env::commit()`.
    type Journal: serde::de::DeserializeOwned;

    /// Parses a `String` as the guest input.
    fn parse_input(&self, input: String) -> Result<Self::Input>;

    /// Encodes the proof into calldata to match the function to call on the Ethereum contract.
    fn encode_calldata(
        &self,
        journal: Self::Journal,
        post_state_digest: FixedBytes<32>,
        seal: Vec<u8>,
    ) -> Result<Vec<u8>>;
}

/// Execute or return image id.
/// If some input is provided, returns the Ethereum ABI and hex encoded proof.
pub fn query(
    guest_list: &[GuestListEntry],
    guest_binary: String,
    input: Option<String>,
    guest_interface: impl GuestInterface,
) -> Result<()> {
    let elf = resolve_guest_entry(guest_list, &guest_binary)?;
    let image_id = compute_image_id(&elf)?;
    let output = match input {
        // Input provided. Return the Ethereum ABI encoded proof.
        Some(input) => {
            let proof = prover::generate_proof(&elf, guest_interface.parse_input(input)?)?;
            hex::encode(proof.abi_encode())
        }
        // No input. Return the Ethereum ABI encoded bytes32 image ID.
        None => format!("0x{}", hex::encode(image_id)),
    };
    print!("{output}");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout buffer")?;
    Ok(())
}

/// Request a proof and publish it on Ethereum.
pub fn publish(
    chain_id: u64,
    eth_wallet_private_key: String,
    rpc_url: String,
    contract: String,
    guest_list: &[GuestListEntry],
    guest_binary: String,
    input: String,
    guest_interface: impl GuestInterface,
) -> Result<()> {
    let elf = resolve_guest_entry(guest_list, &guest_binary)?;
    let tx_sender = eth::TxSender::new(chain_id, &rpc_url, &eth_wallet_private_key, &contract)?;

    let input = guest_interface.parse_input(input)?;
    let Proof {
        journal,
        post_state_digest,
        seal,
    } = prover::generate_proof(&elf, input)?;
    let calldata = guest_interface.encode_calldata(
        risc0_zkvm::serde::from_slice(journal.as_slice())?,
        post_state_digest,
        seal,
    )?;

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(tx_sender.send(calldata))?;

    Ok(())
}

/// Run the CLI.
pub fn run(guest_list: &[GuestListEntry], guest_interface: impl GuestInterface) -> Result<()> {
    match Command::parse() {
        Command::Query {
            guest_binary,
            input,
        } => query(guest_list, guest_binary, input, guest_interface)?,
        Command::Publish {
            chain_id,
            eth_wallet_private_key,
            rpc_url,
            contract,
            guest_binary,
            input,
        } => publish(
            chain_id,
            eth_wallet_private_key,
            rpc_url,
            contract,
            guest_list,
            guest_binary,
            input,
            guest_interface,
        )?,
    }

    Ok(())
}

fn resolve_guest_entry(guest_list: &[GuestListEntry], guest_binary: &String) -> Result<Vec<u8>> {
    // Search list for requested binary name
    let potential_guest_image_id: [u8; 32] =
        match hex::decode(guest_binary.to_lowercase().trim_start_matches("0x")) {
            Ok(byte_vector) => byte_vector.try_into().unwrap_or([0u8; 32]),
            Err(_) => [0u8; 32],
        };
    let guest_entry = guest_list
        .iter()
        .find(|entry| {
            entry.name == guest_binary.to_uppercase()
                || bytemuck::cast::<[u32; 8], [u8; 32]>(entry.image_id) == potential_guest_image_id
        })
        .ok_or_else(|| {
            let found_guests: Vec<String> = guest_list
                .iter()
                .map(|g| hex::encode(bytemuck::cast::<[u32; 8], [u8; 32]>(g.image_id)))
                .collect();
            anyhow!(
                "Unknown guest binary {}, found: {:?}",
                guest_binary,
                found_guests
            )
        })
        .cloned()?;
    Ok(guest_entry.elf.to_vec())
}
