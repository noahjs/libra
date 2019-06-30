use std::sync::Arc;
use std::path::Path;

use failure_ext::prelude::*;
use config::trusted_peers::TrustedPeersConfig;
use types::account_address::AccountAddress;
use types::validator_verifier::ValidatorVerifier;

use crate::grpc_client::GRPCClient;
use crate::client::FaucetClient;

pub struct AppState {
    pub client: GRPCClient,
    // Only needed for minting coins for testnet
    pub faucet_client: FaucetClient,
}

impl AppState {
    pub fn new(
        host: &str,
        ac_port: &str,
        validator_set_file: &str,
        faucet_server: Option<String>,
    ) -> Result<Self> {
        let validators_config = TrustedPeersConfig::load_config(Path::new(validator_set_file));
        let validators = validators_config.get_trusted_consensus_peers();
        
        ensure!(
            !validators.is_empty(),
            "Not able to load validators from trusted peers config!"
        );
        
        // Total 3f + 1 validators, 2f + 1 correct signatures are required.
        // If < 4 validators, all validators have to agree.
        let quorum_size = validators.len() * 2 / 3 + 1;
        let validator_verifier = Arc::new(ValidatorVerifier::new(validators, quorum_size));
        let client = GRPCClient::new(host, ac_port, validator_verifier)?;

        let faucet_url = match faucet_server {
            Some(server) => server.to_string(),
            None => host.replace("ac", "faucet"),
        };

        Ok(AppState {
            client,
            faucet_client: FaucetClient {
                faucet_url,
            },
        })
    }

    pub fn test_validator_connection(&self) -> Result<()> {
        self.client.get_with_proof_sync(vec![])?;
        Ok(())
    }
}
