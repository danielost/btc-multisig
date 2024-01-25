use bdk::bitcoin::network::constants::Network;
use bdk::bitcoin::PrivateKey;
use bdk::blockchain::{Blockchain, ElectrumBlockchain};
use bdk::database::MemoryDatabase;
use bdk::electrum_client::Client;
use bdk::{SyncOptions, Wallet};
use btc_multisig::MultisigWallet;
use std::env;

// This is an example of how to create, sign and send a 2-of-3 multisig transaction
fn main() {
    // Create a wallet using a 2-of-3 multisig descriptor
    let wallet = Wallet::new(
        "sh(multi(2,0330ec812aac4cbc71be2b058a9273a674101b077060e58a70e884db01e3506dfc,0364a819858641c0185d6ba6359712d191e57dcaa5c095c8ceedefb47665480a1e,0235cc22f1ef466270ff652f89eaba49b75527788ae2053b2ba71a0cba73e7dc55))#x02227ng",
        None,
        Network::Testnet,
        MemoryDatabase::new(),
    )
    .expect("failed to instantiate a wallet");

    let client = Client::new("ssl://electrum.blockstream.info:60002").expect("failed to connect");
    let blockchain = ElectrumBlockchain::from(client);

    wallet
        .sync(&blockchain, SyncOptions::default())
        .expect("failed to sync");

    // Import 2 private keys to sign the tx
    let priv_key1: PrivateKey =
        priv_key_from_env("priv_key_1").expect("failed to get priv key 1 from env");
    let priv_key2 = priv_key_from_env("priv_key_2").expect("failed to get priv key 2 from env");

    let mut multisig_wallet = MultisigWallet::new(wallet);
    // Add private keys to the wallet
    multisig_wallet.add_signer(priv_key1);
    multisig_wallet.add_signer(priv_key2);

    let psbt = multisig_wallet
        .create_and_sign_transaction("mnzUjWRB8QEmVXi4q8K8wP8YTScZ5JzQD2", 1000)
        .expect("failed to create and sign transaction");

    let raw_transaction = psbt.extract_tx();
    let txid = raw_transaction.txid();

    blockchain
        .broadcast(&raw_transaction)
        .expect("failed to broadcast transaction");

    println!(
        "Txid: {txid}.\nExplorer URL: https://live.blockcypher.com/btc-testnet/tx/{txid}",
        txid = txid
    );
}

fn priv_key_from_env(name: &str) -> Result<PrivateKey, &'static str> {
    let private_key_wif = match env::var(name) {
        Ok(val) => val,
        Err(_) => return Err("Environment variable not found"),
    };

    let private_key =
        PrivateKey::from_wif(&private_key_wif).map_err(|_| "Failed to decode WIF private key")?;

    Ok(private_key)
}
