use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

pub async fn list(client: &FizzyClient, board_id: &str, json: bool) -> Result<()> {
    if json {
        let raw = client.get_list_raw(&format!("/boards/{board_id}/columns"), true).await?;
        output::print_json(&raw);
    } else {
        let columns: Vec<Column> = client.get_list(&format!("/boards/{board_id}/columns"), true).await?;
        output::print_columns(&columns);
    }
    Ok(())
}

fn color_name_to_var(name: &str) -> String {
    match name.to_lowercase().as_str() {
        "blue" => "var(--color-card-default)".to_string(),
        "gray" | "grey" => "var(--color-card-1)".to_string(),
        "tan" => "var(--color-card-2)".to_string(),
        "yellow" => "var(--color-card-3)".to_string(),
        "lime" => "var(--color-card-4)".to_string(),
        "aqua" => "var(--color-card-5)".to_string(),
        "violet" => "var(--color-card-6)".to_string(),
        "purple" => "var(--color-card-7)".to_string(),
        "pink" => "var(--color-card-8)".to_string(),
        other => other.to_string(), // Pass through if already a CSS var
    }
}

pub async fn create(
    client: &FizzyClient,
    board_id: &str,
    name: &str,
    color: Option<String>,
) -> Result<()> {
    let body = CreateColumnRequest {
        column: CreateColumnBody {
            name: name.to_string(),
            color: color.map(|c| color_name_to_var(&c)),
        },
    };
    client.post_raw(&format!("/boards/{board_id}/columns"), &body).await?;
    println!("Column created.");
    Ok(())
}

pub async fn update(
    client: &FizzyClient,
    board_id: &str,
    column_id: &str,
    name: Option<String>,
    color: Option<String>,
) -> Result<()> {
    let body = UpdateColumnRequest {
        column: UpdateColumnBody {
            name,
            color: color.map(|c| color_name_to_var(&c)),
        },
    };
    client.put(&format!("/boards/{board_id}/columns/{column_id}"), &body).await?;
    println!("Column updated.");
    Ok(())
}

pub async fn delete(client: &FizzyClient, board_id: &str, column_id: &str) -> Result<()> {
    client.delete(&format!("/boards/{board_id}/columns/{column_id}")).await?;
    println!("Column deleted.");
    Ok(())
}
