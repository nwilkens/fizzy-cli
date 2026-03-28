use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

pub async fn list(client: &FizzyClient, card_number: u64, json: bool) -> Result<()> {
    let path = format!("/cards/{card_number}/comments");
    if json {
        let raw = client.get_list_raw(&path, true).await?;
        output::print_json(&raw);
    } else {
        let comments: Vec<Comment> = client.get_list(&path, true).await?;
        output::print_comments(&comments);
    }
    Ok(())
}

pub async fn create(client: &FizzyClient, card_number: u64, body_text: &str) -> Result<()> {
    let body = CreateCommentRequest {
        comment: CreateCommentBody {
            body: body_text.to_string(),
        },
    };
    client.post_raw(&format!("/cards/{card_number}/comments"), &body).await?;
    println!("Comment added to card #{card_number}.");
    Ok(())
}

pub async fn update(client: &FizzyClient, card_number: u64, comment_id: &str, body_text: &str) -> Result<()> {
    let body = UpdateCommentRequest {
        comment: UpdateCommentBody {
            body: body_text.to_string(),
        },
    };
    client.put(&format!("/cards/{card_number}/comments/{comment_id}"), &body).await?;
    println!("Comment updated.");
    Ok(())
}

pub async fn delete(client: &FizzyClient, card_number: u64, comment_id: &str) -> Result<()> {
    client.delete(&format!("/cards/{card_number}/comments/{comment_id}")).await?;
    println!("Comment deleted.");
    Ok(())
}
