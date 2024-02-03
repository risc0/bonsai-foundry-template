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

use std::str::FromStr;

use alloy_primitives::{FixedBytes, U256};
use alloy_sol_types::{sol, SolInterface, SolValue};
use anyhow::Result;
use risc0_ethereum_sdk::cli::GuestInterface;

// You can modify this file to implement the `GuestInterface` trait
// that lets you define how to parse and serialize the guest input and calldata
// so that your contract can interact with the RISC Zero zkVM and Bonsai.

// `IEvenNumber`` interface automatically generated via the alloy `sol!` macro.
// The `set` function is then used as part of the `calldata` function of the
// `EvenNumberInterface`.
sol! {
    interface IEvenNumber {
        function set(uint256 x, bytes32 post_state_digest, bytes calldata seal);
    }
}

/// Implementation of `GuestInterface` for the `EvenNumber` example application.
pub struct EvenNumberInterface;

impl GuestInterface for EvenNumberInterface {
    type Input = U256;

    /// Parses a `String` as the guest input.
    ///
    /// Returned data will be what is read into the guest with `env::read()`.
    fn parse_input(&self, input: String) -> Result<Self::Input> {
        Ok(U256::from_str(&input)?)
    }

    /// Encodes the proof into calldata to match the function to call on the Ethereum contract.
    fn encode_calldata(
        &self,
        journal: Vec<u8>,
        post_state_digest: FixedBytes<32>,
        seal: Vec<u8>,
    ) -> Result<Vec<u8>> {
        // Decode the journal. Must match what was written in the guest with `env::commit_slice`
        let x = U256::abi_decode(&journal, true)?;

        // Encode the function call for `IEvenNumber.set(x)`
        Ok(IEvenNumber::IEvenNumberCalls::set(IEvenNumber::setCall {
            x,
            post_state_digest,
            seal,
        })
        .abi_encode())
    }
}
