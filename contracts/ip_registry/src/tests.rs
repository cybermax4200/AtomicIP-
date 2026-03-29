#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::{Address as _, storage::Persistent, Ledger as _}, Address, BytesN, Env};
    use crate::{IpRegistry, IpRegistryClient, DataKey, LEDGER_BUMP};

    #[test]
    fn test_ttl_extension_after_ip_commit() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IpRegistry, ());
        let client = IpRegistryClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let hash = BytesN::from_array(&env, &[1u8; 32]);
        let ip_id = client.commit_ip(&owner, &hash);

        let ttl = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::IpRecord(ip_id))
        });
        assert!(ttl >= LEDGER_BUMP, "IpRecord TTL must be >= LEDGER_BUMP after commit");
    }

    #[test]
    fn test_ttl_extension_after_owner_ips_update() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IpRegistry, ());
        let client = IpRegistryClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        client.commit_ip(&owner, &BytesN::from_array(&env, &[2u8; 32]));

        let ttl = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::OwnerIps(owner.clone()))
        });
        assert!(ttl >= LEDGER_BUMP, "OwnerIps TTL must be >= LEDGER_BUMP after commit");
    }

    #[test]
    fn test_multiple_ttl_extensions() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(IpRegistry, ());
        let client = IpRegistryClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let ip_id = client.commit_ip(&owner, &BytesN::from_array(&env, &[3u8; 32]));

        // Advance ledger and commit another IP — TTL on NextId must be refreshed.
        env.ledger().set_sequence_number(env.ledger().sequence() + 1000);
        client.commit_ip(&owner, &BytesN::from_array(&env, &[4u8; 32]));

        let ttl = env.as_contract(&contract_id, || {
            env.storage().persistent().get_ttl(&DataKey::IpRecord(ip_id))
        });
        // First record's TTL was set at commit and has not been refreshed — still positive.
        assert!(ttl > 0, "IpRecord TTL must remain positive");
    }
}
