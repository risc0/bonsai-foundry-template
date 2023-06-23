use std::sync::Arc;

use bonsai_proxy_contract::CallbackRequestFilter;
use bonsai_sdk::client::Client;
use ethereum_relay::{
    storage::{
        in_memory::InMemoryStorage, Error as StorageError, ProofRequestState,
        ProofRequstInformation, Storage,
    },
    uploader::bonsai::{
        completed_proofs::manager::BonsaiCompleteProofManager,
        pending_proofs::manager::BonsaiPendingProofManager,
    },
};
use ethers::{
    prelude::Middleware,
    types::{Address, Bytes, Filter, H256},
};
use tokio::sync::Notify;

#[allow(dead_code)]
mod utils;

#[ignore]
#[tokio::test]
async fn integration_test_pending_proof_manager() {
    // Mock API server
    let (proof_id, server) = utils::get_test_bonsai_server().await;

    let client = Client::new(server.uri(), "").unwrap();
    let storage = InMemoryStorage::new();
    let notifier = Arc::new(Notify::new());
    let done_notifer = Arc::new(Notify::new());

    let mut manager = BonsaiPendingProofManager::new(
        client,
        storage.clone(),
        notifier.clone(),
        done_notifer.clone(),
    );

    // add a pending proof request to storage
    storage
        .add_new_bonsai_proof_request(ProofRequstInformation {
            proof_request_id: proof_id,
            callback_proof_request_event: CallbackRequestFilter {
                account: Address::default(),
                image_id: H256::default().into(),
                input: Bytes::default(),
                callback_contract: Address::default(),
                function_selector: [0xab, 0xcd, 0xef, 0xab],
                gas_limit: 3000000,
            },
        })
        .await
        .expect("storage should succeed");

    // now notify the manager
    notifier.notify_one();

    // first step should trigger the notifier and populate the futures stream with a
    // pending proof request
    manager.step().await.expect("step should succeed");
    // second step should complete the pending proof request and trigger the futures
    // stream
    manager.step().await.expect("step should succeed");
    // If we did another manager.step() here it would block since there is no
    // more input for the manager to work on

    done_notifer.notified().await;
}

#[ignore]
#[tokio::test]
async fn integration_test_completed_proof_manager() {
    let anvil = utils::get_anvil();
    let ethers_client = utils::get_ethers_client(
        utils::get_ws_provider(anvil.as_ref()).await,
        utils::get_wallet(anvil.as_ref()),
    )
    .await;

    // Mock API server
    let (proof_id, server) = utils::get_test_bonsai_server().await;

    // deploy the contracts
    let proxy = utils::deploy_proxy_contract(ethers_client.clone()).await;

    let client = Client::new(server.uri(), "").unwrap();
    let storage = InMemoryStorage::new();
    let new_complete_proofs_notifier = Arc::new(Notify::new());
    let send_batch_notifier = Arc::new(Notify::new());
    let max_batch_size: usize = 3;
    // Set some ridicoulous time for the send_batch_interval because we want to
    // control when batches get sent in the test using the send_batch_notifier
    let mut send_batch_interval =
        tokio::time::interval(tokio::time::Duration::from_millis(10000000000));
    // explicitly call tick since the first call to tick will always succeed. By
    // calling it first we can control the flow of the manager in the rest of the
    // test without this interval being triggered.
    send_batch_interval.tick().await;

    let mut manager = BonsaiCompleteProofManager::new(
        client,
        storage.clone(),
        new_complete_proofs_notifier.clone(),
        send_batch_notifier.clone(),
        max_batch_size,
        proxy.address(),
        ethers_client.clone(),
        send_batch_interval,
    );

    // add a complete proof request to storage
    storage
        .add_new_bonsai_proof_request(ProofRequstInformation {
            proof_request_id: proof_id,
            callback_proof_request_event: CallbackRequestFilter {
                account: Address::default(),
                image_id: H256::default().into(),
                input: Bytes::default(),
                callback_contract: Address::default(),
                function_selector: [0xab, 0xcd, 0xef, 0xab],
                gas_limit: 3000000,
            },
        })
        .await
        .expect("storage should succeed");

    // Need to go from Pending -> Completed first
    storage
        .transition_proof_request(proof_id, ProofRequestState::Pending)
        .await
        .expect("should transition to pending");
    storage
        .transition_proof_request(proof_id, ProofRequestState::Completed)
        .await
        .expect("should transition to pending to completed");

    // now notify the manager that a completed proof is ready
    new_complete_proofs_notifier.notify_one();

    // first step should trigger the notifier and populate the futures stream with a
    // bonsai complete proof request. The futures stream will wait until the
    // complete request information is obtained from bonsai
    manager.step().await.expect("step should succeed");
    // The state of the request should now be ProofRequestState::PreparingOnchain
    let request_state = storage
        .get_proof_request_state(proof_id)
        .await
        .expect("proof should exist");
    assert_eq!(request_state, ProofRequestState::PreparingOnchain);

    // second step should add the completed future request to the batch, ready to
    // send
    manager.step().await.expect("step should succeed");

    // now we can signal that the batch should be sent
    send_batch_notifier.notify_one();

    // third step should actually send the batch to the ethereum network
    manager.step().await.expect("step should succeed");

    // check that the event was emitted
    let filter = &Filter::new().address(proxy.address());
    let logs = ethers_client
        .get_logs(&filter)
        .await
        .expect("logs should be present");

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].topics.len(), 1);
    assert_eq!(
        format!("{:?}", logs[0].topics[0]),
        "0xf91ad45be22995db29601925ae62b8fb1c0a2bc3ac736e75866291ad5e6108ce".to_string()
    );

    // verify that the state of the request is CompletedOnchain
    let request_state_response = storage.get_proof_request_state(proof_id).await;
    // The proof request should no longer be in the in-memory database since it is
    // completed
    assert!(request_state_response.is_err());
    assert!(match request_state_response {
        Err(StorageError::ProofNotFound { id }) => id == proof_id,
        _ => false,
    });
}
