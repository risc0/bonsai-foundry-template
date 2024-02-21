// Copyright 2023 RISC Zero, Inc.
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

use std::{collections::HashMap, env, fs, process::Command};

use risc0_build::{embed_methods_with_options, DockerOptions, GuestOptions};
use risc0_zkp::core::digest::Digest;

const SOL_HEADER: &str = r#"// Copyright 2024 RISC Zero, Inc.
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

// This file is automatically generated

"#;

const IMAGE_ID_LIB_HEADER: &str = r#"pragma solidity ^0.8.20;

library ImageID {
"#;

const ELF_LIB_HEADER: &str = r#"pragma solidity ^0.8.20;

library Elf {
"#;

const SOLIDITY_IMAGE_ID_PATH: &str = "../contracts/ImageID.sol";
const SOLIDITY_ELF_PATH: &str = "../tests/Elf.sol";

fn main() {
    let use_docker = env::var("RISC0_USE_DOCKER").ok().map(|_| DockerOptions {
        root_dir: Some("../".into()),
    });

    let methods = embed_methods_with_options(HashMap::from([(
        "bonsai-starter-methods-guest",
        GuestOptions {
            features: Vec::new(),
            use_docker,
        },
    )]));

    let (image_ids, elfs): (Vec<_>, Vec<_>) = methods
        .iter()
        .map(|method| {
            let name = method.name.to_uppercase().replace('-', "_");
            let image_id = hex::encode(Digest::from(method.image_id));
            let image_id_declaration =
                format!("bytes32 public constant {name}_ID = bytes32(0x{image_id});");

            let elf_path = method.path.to_string();
            let elf_declaration = format!("string public constant {name}_PATH = \"{elf_path}\";");

            (image_id_declaration, elf_declaration)
        })
        .unzip();

    let image_ids = image_ids.join("\n");
    let elfs = elfs.join("\n");

    // Building the final image_ID file content.
    let file_content = format!("{SOL_HEADER}{IMAGE_ID_LIB_HEADER}\n{image_ids}\n}}");
    fs::write(SOLIDITY_IMAGE_ID_PATH, file_content).unwrap_or_else(|err| {
        panic!(
            "failed to save changes to {}: {}",
            SOLIDITY_IMAGE_ID_PATH, err
        );
    });

    // Building the final elf file content.
    let file_content = format!("{SOL_HEADER}{ELF_LIB_HEADER}\n{elfs}\n}}");
    fs::write(SOLIDITY_ELF_PATH, file_content).unwrap_or_else(|err| {
        panic!("failed to save changes to {}: {}", SOLIDITY_ELF_PATH, err);
    });

    // use `forge fmt` to format the generated code
    Command::new("forge")
        .arg("fmt")
        .arg(SOLIDITY_IMAGE_ID_PATH)
        .arg(SOLIDITY_ELF_PATH)
        .status()
        .unwrap_or_else(|e| {
            panic!("failed to format {SOLIDITY_IMAGE_ID_PATH}, {SOLIDITY_ELF_PATH}: {e}")
        });
}
