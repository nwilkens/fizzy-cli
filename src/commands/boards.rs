use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

pub async fn list(client: &FizzyClient, json: bool) -> Result<()> {
    if json {
        let raw = client.get_list_raw("/boards", true).await?;
        output::print_json(&raw);
    } else {
        let boards: Vec<Board> = client.get_list("/boards", true).await?;
        output::print_boards(&boards);
    }
    Ok(())
}

pub async fn show(client: &FizzyClient, id: &str, json: bool) -> Result<()> {
    if json {
        let raw = client.get_raw(&format!("/boards/{id}")).await?;
        output::print_json(&raw);
    } else {
        let board: Board = client.get(&format!("/boards/{id}")).await?;
        output::print_board_detail(&board);
    }
    Ok(())
}

pub async fn create(
    client: &FizzyClient,
    name: &str,
    all_access: Option<bool>,
    entropy: Option<u32>,
    json: bool,
) -> Result<()> {
    let body = CreateBoardRequest {
        board: CreateBoardBody {
            name: name.to_string(),
            all_access,
            auto_postpone_period_in_days: entropy,
        },
    };
    let raw = client.post_raw("/boards", &body).await?;
    if json {
        output::print_json(&raw);
    } else {
        println!("Board created.");
    }
    Ok(())
}

pub async fn update(
    client: &FizzyClient,
    id: &str,
    name: Option<String>,
    all_access: Option<bool>,
    entropy: Option<u32>,
) -> Result<()> {
    let body = UpdateBoardRequest {
        board: UpdateBoardBody {
            name,
            all_access,
            auto_postpone_period_in_days: entropy,
        },
    };
    client.put(&format!("/boards/{id}"), &body).await?;
    println!("Board updated.");
    Ok(())
}

pub async fn delete(client: &FizzyClient, id: &str) -> Result<()> {
    client.delete(&format!("/boards/{id}")).await?;
    println!("Board deleted.");
    Ok(())
}

pub async fn publish(client: &FizzyClient, id: &str, json: bool) -> Result<()> {
    let raw = client.post_raw(&format!("/boards/{id}/publication"), &serde_json::json!({})).await?;
    if json {
        output::print_json(&raw);
    } else {
        println!("Board published.");
    }
    Ok(())
}

pub async fn unpublish(client: &FizzyClient, id: &str) -> Result<()> {
    client.delete(&format!("/boards/{id}/publication")).await?;
    println!("Board unpublished.");
    Ok(())
}
