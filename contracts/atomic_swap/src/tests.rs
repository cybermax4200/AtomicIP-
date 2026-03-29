#[cfg(test)]
mod tests {
    use ip_registry::{IpRegistry, IpRegistryClient};
    use soroban_sdk::{testutils::{Address as _, storage::Persistent}, Address, BytesN, Env};
    use soroban_sdk::token::StellarAssetClient;
    use crate::{AtomicSwap, AtomicSwapClient, DataKey, LEDGER_BUMP};

    fn setup_registry_with_ip(env: &Env, owner: &Address) -> (Address, u64) {
        let registry_id = env.register(IpRegistry, ());
        let registry = IpRegistryClient::new(env, &registry_id);
        let ip_id = registry.commit_ip(owner, &BytesN::from_array(env, &[1u8; 32]));
        (registry_id, ip_id)
    }

    fn setup_token(env: &Env, admin: &Address, recipient: &Address, amount: i128) -> Address {
        let token_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
        StellarAssetClient::new(env, &token_id).mint(recipient, &amount);
        token_id
    }

    #[test]
    fn test_ttl_extension_after_swap_initiation() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);
        let admin = Address::generate(&env);
        let (registry_id, ip_id) = setup_registry_with_ip(&env, &seller);
        let token_id = setup_token(&env, &admin, &buyer, 1000);

        let contract_id = env.register(AtomicSwap, ());
        let client = AtomicSwapClient::new(&env, &contract_id);
        let swap_id = client.initiate_swap(&registry_id, &token_id, &ip_id, &seller, &100_i128, &buyer);

        let swap_ttl = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::Swap(swap_id))
        });
        let active_ttl = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::ActiveSwap(ip_id))
        });
        assert!(swap_ttl >= LEDGER_BUMP, "Swap TTL must be >= LEDGER_BUMP after initiation");
        assert!(active_ttl >= LEDGER_BUMP, "ActiveSwap TTL must be >= LEDGER_BUMP after initiation");
    }

    #[test]
    fn test_ttl_extension_after_swap_acceptance() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);
        let admin = Address::generate(&env);
        let (registry_id, ip_id) = setup_registry_with_ip(&env, &seller);
        let token_id = setup_token(&env, &admin, &buyer, 1000);

        let contract_id = env.register(AtomicSwap, ());
        let client = AtomicSwapClient::new(&env, &contract_id);
        let swap_id = client.initiate_swap(&registry_id, &token_id, &ip_id, &seller, &100_i128, &buyer);
        client.accept_swap(&swap_id);

        let ttl = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::Swap(swap_id))
        });
        assert!(ttl >= LEDGER_BUMP, "Swap TTL must be >= LEDGER_BUMP after acceptance");
    }

    #[test]
    fn test_ttl_extension_after_swap_completion() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);
        let admin = Address::generate(&env);

        // Build a real commitment so reveal_key passes verification.
        let secret = BytesN::from_array(&env, &[7u8; 32]);
        let blinding = BytesN::from_array(&env, &[8u8; 32]);
        let mut preimage = soroban_sdk::Bytes::new(&env);
        preimage.append(&soroban_sdk::Bytes::from(secret.clone()));
        preimage.append(&soroban_sdk::Bytes::from(blinding.clone()));
        let commitment_hash: BytesN<32> = env.crypto().sha256(&preimage).into();

        let registry_id = env.register(IpRegistry, ());
        let registry = IpRegistryClient::new(&env, &registry_id);
        let ip_id = registry.commit_ip(&seller, &commitment_hash);

        let token_id = setup_token(&env, &admin, &buyer, 1000);
        let contract_id = env.register(AtomicSwap, ());
        let client = AtomicSwapClient::new(&env, &contract_id);

        let swap_id = client.initiate_swap(&registry_id, &token_id, &ip_id, &seller, &1000_i128, &buyer);
        client.accept_swap(&swap_id);
        client.reveal_key(&swap_id, &seller, &secret, &blinding);

        let ttl = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::Swap(swap_id))
        });
        assert!(ttl >= LEDGER_BUMP, "Swap TTL must be >= LEDGER_BUMP after completion");
    }

    #[test]
    fn test_ttl_extension_after_swap_cancellation() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);
        let admin = Address::generate(&env);
        let (registry_id, ip_id) = setup_registry_with_ip(&env, &seller);
        let token_id = setup_token(&env, &admin, &buyer, 1000);

        let contract_id = env.register(AtomicSwap, ());
        let client = AtomicSwapClient::new(&env, &contract_id);
        let swap_id = client.initiate_swap(&registry_id, &token_id, &ip_id, &seller, &100_i128, &buyer);
        client.cancel_swap(&swap_id, &seller);

        let ttl = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::Swap(swap_id))
        });
        assert!(ttl >= LEDGER_BUMP, "Swap TTL must be >= LEDGER_BUMP after cancellation");
    }

    #[test]
    fn test_multiple_ttl_extensions_during_swap_lifecycle() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);
        let admin = Address::generate(&env);

        let secret = BytesN::from_array(&env, &[9u8; 32]);
        let blinding = BytesN::from_array(&env, &[10u8; 32]);
        let mut preimage = soroban_sdk::Bytes::new(&env);
        preimage.append(&soroban_sdk::Bytes::from(secret.clone()));
        preimage.append(&soroban_sdk::Bytes::from(blinding.clone()));
        let commitment_hash: BytesN<32> = env.crypto().sha256(&preimage).into();

        let registry_id = env.register(IpRegistry, ());
        let registry = IpRegistryClient::new(&env, &registry_id);
        let ip_id = registry.commit_ip(&seller, &commitment_hash);

        let token_id = setup_token(&env, &admin, &buyer, 1000);
        let contract_id = env.register(AtomicSwap, ());
        let client = AtomicSwapClient::new(&env, &contract_id);

        let swap_id = client.initiate_swap(&registry_id, &token_id, &ip_id, &seller, &1000_i128, &buyer);
        let ttl_init = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::Swap(swap_id))
        });

        client.accept_swap(&swap_id);
        let ttl_accept = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::Swap(swap_id))
        });

        client.reveal_key(&swap_id, &seller, &secret, &blinding);
        let ttl_complete = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::Swap(swap_id))
        });

        assert!(ttl_init >= LEDGER_BUMP);
        assert!(ttl_accept >= LEDGER_BUMP);
        assert!(ttl_complete >= LEDGER_BUMP);
    }
}
