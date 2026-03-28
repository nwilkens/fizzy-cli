use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

pub async fn list(client: &FizzyClient, json: bool) -> Result<()> {
    if json {
        let raw = client.get_list_raw("/users", true).await?;
        output::print_json(&raw);
    } else {
        let users: Vec<User> = client.get_list("/users", true).await?;
        output::print_users(&users);
    }
    Ok(())
}

pub async fn show(client: &FizzyClient, id: &str, json: bool) -> Result<()> {
    if json {
        let raw = client.get_raw(&format!("/users/{id}")).await?;
        output::print_json(&raw);
    } else {
        let user: User = client.get(&format!("/users/{id}")).await?;
        output::print_user_detail(&user);
    }
    Ok(())
}
