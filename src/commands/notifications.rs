use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

pub async fn list(client: &FizzyClient, json: bool) -> Result<()> {
    if json {
        let raw = client.get_list_raw("/notifications", true).await?;
        output::print_json(&raw);
    } else {
        let notifications: Vec<Notification> = client.get_list("/notifications", true).await?;
        output::print_notifications(&notifications);
    }
    Ok(())
}

pub async fn read_all(client: &FizzyClient) -> Result<()> {
    client.post_no_body("/notifications/bulk_reading").await?;
    println!("All notifications marked as read.");
    Ok(())
}

pub async fn mark_read(client: &FizzyClient, id: &str) -> Result<()> {
    client.post_no_body(&format!("/notifications/{id}/reading")).await?;
    println!("Notification marked as read.");
    Ok(())
}

pub async fn mark_unread(client: &FizzyClient, id: &str) -> Result<()> {
    client.delete(&format!("/notifications/{id}/reading")).await?;
    println!("Notification marked as unread.");
    Ok(())
}
