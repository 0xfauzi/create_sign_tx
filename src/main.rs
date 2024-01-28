use std::str::FromStr;
use bdk::{bitcoin, descriptor, FeeRate, SignOptions, SyncOptions, Wallet};
use bdk::bitcoin::{Address, key, Network, PrivateKey};
use bdk::bitcoin::Network::Testnet;
use bdk::blockchain::{Blockchain, ElectrumBlockchain};
use bdk::database::MemoryDatabase;
use bdk::electrum_client::Client;
use bdk::wallet::AddressIndex::New;

fn main() {
    let secp = bitcoin::secp256k1::Secp256k1::new();

    // testNet generated private key for demo purposes
    let private_key = PrivateKey::from_wif("cNR1rFRvrXU6UWrhvfrpwtoG7zLdTEzgAfMcZifaF5vDgiDPrVtX").unwrap();
    let public_key = key::PublicKey::from_private_key(&secp, &private_key);

    // Create a descriptor
    let descriptor = descriptor::Descriptor::new_pkh(public_key).unwrap().to_string();

    let client = Client::new("ssl://electrum.blockstream.info:60002").unwrap();
    let wallet = Wallet::new(
        &descriptor,
        Some(&descriptor),
        Testnet,
        MemoryDatabase::new(),
    ).unwrap();

    let blockchain = ElectrumBlockchain::from(client);
    wallet.sync(&blockchain, SyncOptions::default()).expect("Wallet did not sync");

    let address = wallet.get_address(New).unwrap();
    println!("Generated Address: {}", address);

    let balance = wallet.get_balance().unwrap();
    println!("Balance: {}", balance.to_string());

    let send_to = Address::from_str("mv4rnyY3Su5gjcDNzbMLKBQkBicCtHUtFB").unwrap().require_network(Testnet).unwrap();
    let (mut psbt, details) = {
        let mut builder =  wallet.build_tx();
        builder
            .add_recipient(send_to.script_pubkey(), 50_000)
            .enable_rbf()
            .fee_rate(FeeRate::from_sat_per_vb(5.0));
        builder.finish().unwrap()
    };

    print!("Tx details: {:?}", details);

    // Sign transaction
    wallet.sign(&mut psbt, SignOptions::default()).expect("Transaction couldn't be finalized");

    // Serialize transaction
    let raw_transaction = &mut psbt.extract_tx();
    let serialized_tx = bitcoin::consensus::encode::serialize(&raw_transaction);

    println!("Signed Transaction: {:?}", serialized_tx);
    println!("Transaction details: {:#?}", details);
}
