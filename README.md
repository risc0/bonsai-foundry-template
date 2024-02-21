# Bonsai Foundry Template

> **Note: This software is not production ready. Do not use in production.**

Starter template for writing an application using [RISC Zero] and Ethereum.

This repository implements an application on Ethereum utilizing RISC Zero as a [coprocessor] to the smart contract application.
It provides a starting point for building powerful new applications on Ethereum that offload computationally intensive (i.e. gas expensive), or difficult to implement.
Prove computation with the [RISC Zero zkVM] and verify the results in your Ethereum contract.

## Overview

Here is a simplified overview of how devs can integrate RISC Zero, with [Bonsai] proving, into their Ethereum smart contracts:

![Bonsai Foundry Template Diagram](images/bonsai-foundry-template.png)

1. Run your application logic in the [RISC Zero zkVM]. The provided [publisher] app sends an off-chain proof request to the [Bonsai] proving service.
2. [Bonsai] generates the program result, written to the [journal], and a SNARK proof of its correctness.
3. The [publisher] app submits this proof and journal on-chain to your app contract for validation.
4. Your app contract calls the [RISC Zero Verifier] to validate the proof. If the verification is successful, the journal is deemed trustworthy and can be safely used.

## Dependencies

First, [install Rust] and [Foundry], and then restart your terminal.

```sh
# Install Rust
curl https://sh.rustup.rs -sSf | sh
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
```

Next, you will need to install the `cargo risczero` tool.
We'll use [`cargo binstall`][cargo-binstall] to get `cargo-risczero` installed. See [RISC Zero installation] for more details.

```sh
cargo install cargo-binstall
cargo binstall cargo-risczero
```

Next we'll need to install the `risc0` toolchain with:

```sh
cargo risczero install
```

Now you have all the tools you need to develop and deploy an application with [RISC Zero].

## Quick Start

First, install the RISC Zero toolchain using the [instructions above].

Now, you can initialize a new Bonsai project at a location of your choosing:

```sh
forge init -t risc0/bonsai-foundry-template ./my-project
```

Congratulations! You've just started your first RISC Zero project.

Your new project consists of:

- a [zkVM program] (written in Rust), which specifies a computation that will be proven;
- a [app contract] (written in Solidity), which receives the response;
- a [guest interface] (written in Rust), which lets you define how to parse and serialize the guest input and calldata so that the [RISC Zero zkVM] can integrate with your contract.

### Run the Tests

- Use `cargo test` to run the tests in your zkVM program.
- Use `RISC0_DEV_MODE=true forge test -vvv` to test your Solidity contracts and their interaction with your zkVM program.

## Develop Your Application

To build your application, you'll need to make changes in three folders:

- write the code you want proven in the [methods] folder
- write the on-chain part of your project in the [contracts] folder
- write the guest interface in the [cli] folder

### Configuring Bonsai

***Note:*** *The Bonsai proving service is still in early Alpha. To request an API key [complete the form here](https://bonsai.xyz/apply).*

With the Bonsai proving service, you can produce a [Groth16 SNARK proof] that is verifiable on-chain.
You can get started by setting the following environment variables with your API key and associated URL.

```bash
export BONSAI_API_KEY="YOUR_API_KEY" # see form linked above
export BONSAI_API_URL="BONSAI_URL" # provided with your api key
```

Now if you run `forge test` with `RISC0_DEV_MODE=false`, the test will run as before, but will additionally use the fully verifying `RiscZeroGroth16Verifier` contract instead of `MockRiscZeroVerifier` and will request a SNARK receipt from Bonsai.

```bash
RISC0_DEV_MODE=false forge test -vvv
```

## Deploy Your Application

When you're ready, follow the [deployment guide] to get your application running on [Sepolia].

## Project Structure

Below are the primary files in the project directory

```text
.
├── Cargo.toml                      // Configuration for Cargo and Rust
├── foundry.toml                    // Configuration for Foundry
├── apps
│   ├── Cargo.toml
│   └── src
│       └── lib.rs                  // Utility functions
│       └── bin                     
│           └── publisher.rs        // Example app to publish program results into your app contract 
├── contracts
│   └── EvenNumber.sol              // Basic example contract for you to modify
├── methods
│   ├── Cargo.toml
│   ├── guest
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── bin                 // You can add additionally guest prgrams to this folder
│   │           └── is_even.rs      // Example guest program for cheking if a number is even
│   └── src
│       └── lib.rs                  // Compiled image IDs and tests for your guest programs
└── tests
    ├── EvenNumber.t.sol            // Tests for the basic example contract
    └── MockRiscZeroVerifier.sol    // RISC Zero Verifier mock contract
```

[RISC Zero]: https://www.risczero.com/
[Bonsai]: https://dev.bonsai.xyz/
[coprocessor]: https://twitter.com/RiscZero/status/1677316664772132864
[RISC Zero zkVM]: https://dev.risczero.com/zkvm
[journal]: https://dev.risczero.com/terminology#journal
[RISC Zero Verifier]: https://github.com/risc0/risc0/blob/release-0.20/bonsai/ethereum/contracts/IRiscZeroVerifier.sol
[install Rust]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[Foundry]: https://getfoundry.sh/
[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall#cargo-binaryinstall
[instructions above]: #dependencies
[zkVM program]: ./methods/guest/src/bin
[app contract]: ./contracts
[guest interface]: ./cli
[methods]: ./methods/
[cli]: ./cli/
[contracts]: ./contracts/
[Groth16 SNARK proof]: https://www.risczero.com/news/on-chain-verification
[deployment guide]: /deployment-guide.md
[Sepolia]: https://www.alchemy.com/overviews/sepolia-testnet
[RISC Zero installation]: https://dev.risczero.com/api/zkvm/install
[publisher]: ./apps/README.md
