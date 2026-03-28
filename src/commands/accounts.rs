use anyhow::Result;

use crate::client::FizzyClient;
use crate::config::Config;
use crate::models::IdentityResponse;
use crate::output;

pub async fn list(config: &Config, url_override: Option<&str>, json: bool) -> Result<()> {
    let client = FizzyClient::new_unscoped(config, url_override)?;

    if json {
        let raw: serde_json::Value = client.get_global_raw("/my/identity").await?;
        output::print_json(&raw);
    } else {
        let identity: IdentityResponse = client.get_global("/my/identity").await?;
        output::print_accounts(&identity.accounts);
    }
    Ok(())
}
