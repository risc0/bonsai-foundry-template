use bonsai_ethereum_relay::sdk::client::{CallbackRequest, Client};
use clap::Parser;
use ethers::{
    abi::{ethabi, Uint},
    types::Address,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    image_id: String,
    address: Address,
    number: Uint,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    // initialize a relay client
    let relay_client = Client::from_parts(
        "http://localhost:8080".to_string(), // here goes the actual url of the Bonsai Relay
        "BONSAI_API_KEY".to_string(),        // here goes the actual Bonsai API-Key
    )
    .expect("Failed to initialize the relay client");

    // Initialize the input for the FIBONACCI guest.
    let input = ethabi::encode(&[ethers::abi::Token::Uint(args.number)]);

    // Create a CallbackRequest for your contract
    // example: (contracts/BonsaiStarter.sol).
    let image_id: [u8; 32] = hex::decode(args.image_id)
        .expect("unable to decode")
        .try_into()
        .expect("unable to convert image id");
    let request = CallbackRequest {
        callback_contract: args.address,
        // you can use the command `solc --hashes contracts/BonsaiStarter.sol`
        // to get the value for your actual contract
        function_selector: [0xe8, 0x08, 0x02, 0xa2],
        gas_limit: 3000000,
        image_id: image_id.into(),
        input,
    };

    // Send the callback request to the Bonsai Relay.
    relay_client
        .callback_request(request)
        .await
        .expect("Callback request failed");
}
