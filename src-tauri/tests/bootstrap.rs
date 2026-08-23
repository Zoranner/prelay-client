use prelay_client::{
    api_client::ApiClient,
    commands::bootstrap::collect_bootstrap,
    credential_store::MemoryCredentialStore,
    identity::{IdentitySource, WindowsIdentity},
};

struct FakeWindowsIdentity {
    identity: WindowsIdentity,
}

impl FakeWindowsIdentity {
    fn new(machine_id: &str, account_sid: &str) -> Self {
        Self {
            identity: WindowsIdentity {
                machine_id: machine_id.into(),
                account_sid: account_sid.into(),
                display_name: "Ada".into(),
            },
        }
    }
}

impl IdentitySource for FakeWindowsIdentity {
    fn identity(&self) -> Result<WindowsIdentity, String> {
        Ok(self.identity.clone())
    }
}

#[test]
fn bootstrap_only_exposes_display_identity_and_credential_status() {
    let identity = FakeWindowsIdentity::new("machine-a", "S-1-5-21-100");
    let credentials = MemoryCredentialStore::with_record("device-secret", None);
    let api_client =
        ApiClient::new("https://relay.example.test", &credentials).expect("create API client");

    let response = collect_bootstrap(&identity, &api_client).unwrap();

    assert_eq!(response.display_name, "Ada");
    assert_eq!(response.avatar_seed, "3f1bf7a2cfc2b426");
    assert_eq!(response.relay_url, "https://relay.example.test");
    assert!(response.has_device_credential);
    let response = serde_json::to_value(response).unwrap();
    assert!(response.get("machine_id").is_none());
    assert!(response.get("account_sid").is_none());
    assert!(response.get("avatar_seed").is_some());
    assert!(response.get("device_credential").is_none());
}
