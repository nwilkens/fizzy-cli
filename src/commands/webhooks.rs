use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

pub async fn list(client: &FizzyClient, board_id: &str, json: bool) -> Result<()> {
    let path = format!("/boards/{board_id}/webhooks");
    if json {
        let raw = client.get_list_raw(&path, true).await?;
        output::print_json(&raw);
    } else {
        let webhooks: Vec<Webhook> = client.get_list(&path, true).await?;
        output::print_webhooks(&webhooks);
    }
    Ok(())
}

pub async fn show(client: &FizzyClient, board_id: &str, id: &str, json: bool) -> Result<()> {
    let path = format!("/boards/{board_id}/webhooks/{id}");
    if json {
        let raw = client.get_raw(&path).await?;
        output::print_json(&raw);
    } else {
        let webhook: Webhook = client.get(&path).await?;
        output::print_webhook_detail(&webhook);
    }
    Ok(())
}

pub async fn create(
    client: &FizzyClient,
    board_id: &str,
    name: &str,
    url: &str,
    actions: &str,
    json: bool,
) -> Result<()> {
    let subscribed_actions: Vec<String> = actions.split(',').map(|s| s.trim().to_string()).collect();
    let body = CreateWebhookRequest {
        webhook: CreateWebhookBody {
            name: name.to_string(),
            url: url.to_string(),
            subscribed_actions,
        },
    };
    let raw = client.post_raw(&format!("/boards/{board_id}/webhooks"), &body).await?;
    if json {
        output::print_json(&raw);
    } else {
        println!("Webhook created.");
    }
    Ok(())
}

pub async fn update(
    client: &FizzyClient,
    board_id: &str,
    id: &str,
    name: Option<String>,
    actions: Option<String>,
) -> Result<()> {
    let body = UpdateWebhookRequest {
        webhook: UpdateWebhookBody {
            name,
            subscribed_actions: actions
                .map(|a| a.split(',').map(|s| s.trim().to_string()).collect()),
        },
    };
    client.patch(&format!("/boards/{board_id}/webhooks/{id}"), &body).await?;
    println!("Webhook updated.");
    Ok(())
}

pub async fn delete(client: &FizzyClient, board_id: &str, id: &str) -> Result<()> {
    client.delete(&format!("/boards/{board_id}/webhooks/{id}")).await?;
    println!("Webhook deleted.");
    Ok(())
}

pub async fn activate(client: &FizzyClient, board_id: &str, id: &str) -> Result<()> {
    client.post_no_body(&format!("/boards/{board_id}/webhooks/{id}/activation")).await?;
    println!("Webhook activated.");
    Ok(())
}
