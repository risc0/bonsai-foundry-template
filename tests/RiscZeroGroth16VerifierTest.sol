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
//
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.9;

import {SafeCast} from "openzeppelin/contracts/utils/math/SafeCast.sol";

import {Groth16Verifier} from "bonsai/groth16/Groth16Verifier.sol";
import {
    ExitCode,
    IRiscZeroVerifier,
    Output,
    OutputLib,
    Receipt,
    ReceiptClaim,
    ReceiptClaimLib,
    SystemExitCode
} from "bonsai/IRiscZeroVerifier.sol";

/// @notice Groth16 verifier contract for RISC Zero receipts of execution.
contract RiscZeroGroth16VerifierTest is IRiscZeroVerifier, Groth16Verifier {
    uint256 public immutable CONTROL_ID_0;
    uint256 public immutable CONTROL_ID_1;

    constructor() {
        CONTROL_ID_0 = 0;
        CONTROL_ID_1 = 0;
    }

    /// @inheritdoc IRiscZeroVerifier
    function verify(
        bytes calldata seal,
        bytes32,
        /*imageId*/
        bytes32 postStateDigest,
        bytes32 /*journalDigest*/
    ) public view returns (bool) {
        // Require that the seal be specifically empty.
        // Reject if the caller may have sent a real seal.
        return CONTROL_ID_0 == 0 && CONTROL_ID_1 == 0 && seal.length == 0 && postStateDigest == bytes32(0);
    }

    /// @inheritdoc IRiscZeroVerifier
    function verify_integrity(Receipt memory /*receipt*/ ) public view returns (bool) {
        return CONTROL_ID_0 == 0 && CONTROL_ID_1 == 0;
    }
}