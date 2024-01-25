use std::{str::FromStr, sync::Arc};

use bdk::{
    bitcoin::{psbt::PartiallySignedTransaction, Address, PrivateKey},
    database::BatchDatabase,
    signer::{SignerOrdering, SignerWrapper},
    SignOptions, Wallet,
};

pub struct MultisigWallet<D> {
    multisig_wallet: Wallet<D>,
}

impl<D> MultisigWallet<D>
where
    D: BatchDatabase,
{
    pub fn new(wallet: Wallet<D>) -> Self {
        MultisigWallet {
            multisig_wallet: wallet,
        }
    }

    pub fn add_signer(&mut self, signer: PrivateKey) {
        let signer_wrapper = SignerWrapper::new(signer, bdk::signer::SignerContext::Legacy);
        self.multisig_wallet.add_signer(
            bdk::KeychainKind::External,
            SignerOrdering::default(),
            Arc::from(signer_wrapper),
        );
    }

    pub fn create_and_sign_transaction(
        &mut self,
        recipient_address: &str,
        amount: u64,
    ) -> Result<PartiallySignedTransaction, Box<dyn std::error::Error>> {
        let address = Address::from_str(recipient_address)?;
        let mut builder = self.multisig_wallet.build_tx();
        builder
            .add_recipient(address.payload.script_pubkey(), amount)
            .enable_rbf();

        let (mut psbt, _) = builder.finish()?;

        self.multisig_wallet
            .sign(&mut psbt, SignOptions::default())?;

        Ok(psbt)
    }
}
