use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

pub async fn list(client: &FizzyClient, json: bool) -> Result<()> {
    if json {
        let raw = client.get_list_raw("/tags", true).await?;
        output::print_json(&raw);
    } else {
        let tags: Vec<Tag> = client.get_list("/tags", true).await?;
        output::print_tags(&tags);
    }
    Ok(())
}
