use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;

pub async fn add(client: &FizzyClient, card_number: u64, content: &str) -> Result<()> {
    let body = CreateStepRequest {
        step: CreateStepBody {
            content: content.to_string(),
            completed: None,
        },
    };
    client.post_raw(&format!("/cards/{card_number}/steps"), &body).await?;
    println!("Step added to card #{card_number}.");
    Ok(())
}

pub async fn complete(client: &FizzyClient, card_number: u64, step_id: &str) -> Result<()> {
    let body = UpdateStepRequest {
        step: UpdateStepBody {
            content: None,
            completed: Some(true),
        },
    };
    client.put(&format!("/cards/{card_number}/steps/{step_id}"), &body).await?;
    println!("Step completed.");
    Ok(())
}

pub async fn delete(client: &FizzyClient, card_number: u64, step_id: &str) -> Result<()> {
    client.delete(&format!("/cards/{card_number}/steps/{step_id}")).await?;
    println!("Step deleted.");
    Ok(())
}
