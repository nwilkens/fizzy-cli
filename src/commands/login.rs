use anyhow::Result;
use dialoguer::Input;

use crate::client::FizzyClient;
use crate::config::Config;
use crate::models::IdentityResponse;
use crate::output;

pub async fn login(token: Option<String>, url_override: Option<&str>, json: bool) -> Result<()> {
    let mut config = Config::load()?;
    let base_url = url_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| config.base_url());

    let final_token = if let Some(t) = token {
        // Token paste flow
        t
    } else {
        // Magic link flow
        let email: String = Input::new()
            .with_prompt("Email")
            .interact_text()?;

        let client = FizzyClient::unauthenticated(&base_url)?;
        let pending_token = client.request_magic_link(&email).await?;
        println!("Magic link sent! Check your email.");

        let code: String = Input::new()
            .with_prompt("Code")
            .interact_text()?;

        let session_token = client.submit_magic_link_code(&code, &pending_token).await?;
        println!("Authenticated. Creating access token...");

        // We need to get identity to find an account slug for token creation
        // First, use session cookie to list identity
        // For now, we create a temporary client with the session to get identity
        // The /my/identity endpoint is global (no account needed)
        // But creating an access token requires an account scope
        // So we need to fetch identity first to get an account slug

        // Use a raw reqwest call with session cookie to get identity
        let http = reqwest::Client::new();
        let resp = http
            .get(format!("{}/my/identity", base_url))
            .header("Accept", "application/json")
            .header("Cookie", format!("session_token={session_token}"))
            .header("User-Agent", "fz/0.1.0")
            .send()
            .await?;
        let identity: IdentityResponse = resp.json().await?;

        if identity.accounts.is_empty() {
            anyhow::bail!("No accounts found for this identity.");
        }

        let slug = identity.accounts[0].slug.trim_start_matches('/');
        let token_resp = client
            .create_access_token_with_session(&session_token, slug)
            .await?;

        println!("Access token created.");
        token_resp.token
    };

    // Verify the token
    let client = FizzyClient::with_token(&base_url, &final_token)?;
    let identity: IdentityResponse = client.get_global("/my/identity").await?;

    if json {
        let raw: serde_json::Value = client.get_global_raw("/my/identity").await?;
        output::print_json(&raw);
    } else {
        println!("Logged in successfully!\n");
        output::print_accounts(&identity.accounts);
    }

    // Save config
    config.token = Some(final_token);
    if url_override.is_some() {
        config.base_url = Some(base_url);
    }

    // Auto-select account
    if identity.accounts.len() == 1 {
        let slug = identity.accounts[0].slug.trim_start_matches('/').to_string();
        config.account = Some(slug.clone());
        if !json {
            println!("\nDefault account set to: {} ({})", identity.accounts[0].name, slug);
        }
    } else if identity.accounts.len() > 1 && config.account.is_none() {
        let slug = identity.accounts[0].slug.trim_start_matches('/').to_string();
        config.account = Some(slug.clone());
        if !json {
            println!(
                "\nDefault account set to: {} ({})\nTo change: fz set account <slug>",
                identity.accounts[0].name, slug
            );
        }
    }

    config.save()?;
    Ok(())
}

pub async fn logout() -> Result<()> {
    let mut config = Config::load()?;
    config.token = None;
    config.save()?;
    println!("Logged out. Token removed from config.");
    Ok(())
}
