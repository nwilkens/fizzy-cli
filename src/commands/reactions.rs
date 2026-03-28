use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

// Card reactions
pub async fn list_card(client: &FizzyClient, card_number: u64, json: bool) -> Result<()> {
    let path = format!("/cards/{card_number}/reactions");
    if json {
        let raw = client.get_list_raw(&path, true).await?;
        output::print_json(&raw);
    } else {
        let reactions: Vec<Reaction> = client.get_list(&path, true).await?;
        output::print_reactions(&reactions);
    }
    Ok(())
}

pub async fn create_card(client: &FizzyClient, card_number: u64, content: &str) -> Result<()> {
    let body = CreateReactionRequest {
        reaction: CreateReactionBody {
            content: content.to_string(),
        },
    };
    client.post_raw(&format!("/cards/{card_number}/reactions"), &body).await?;
    println!("Reaction added to card #{card_number}.");
    Ok(())
}

pub async fn delete_card(client: &FizzyClient, card_number: u64, reaction_id: &str) -> Result<()> {
    client.delete(&format!("/cards/{card_number}/reactions/{reaction_id}")).await?;
    println!("Reaction removed.");
    Ok(())
}

// Comment reactions
pub async fn list_comment(client: &FizzyClient, card_number: u64, comment_id: &str, json: bool) -> Result<()> {
    let path = format!("/cards/{card_number}/comments/{comment_id}/reactions");
    if json {
        let raw = client.get_list_raw(&path, true).await?;
        output::print_json(&raw);
    } else {
        let reactions: Vec<Reaction> = client.get_list(&path, true).await?;
        output::print_reactions(&reactions);
    }
    Ok(())
}

pub async fn create_comment(client: &FizzyClient, card_number: u64, comment_id: &str, content: &str) -> Result<()> {
    let body = CreateReactionRequest {
        reaction: CreateReactionBody {
            content: content.to_string(),
        },
    };
    client.post_raw(&format!("/cards/{card_number}/comments/{comment_id}/reactions"), &body).await?;
    println!("Reaction added to comment.");
    Ok(())
}

pub async fn delete_comment(client: &FizzyClient, card_number: u64, comment_id: &str, reaction_id: &str) -> Result<()> {
    client.delete(&format!("/cards/{card_number}/comments/{comment_id}/reactions/{reaction_id}")).await?;
    println!("Reaction removed from comment.");
    Ok(())
}
