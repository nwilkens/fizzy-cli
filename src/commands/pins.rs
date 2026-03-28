use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

pub async fn list(client: &FizzyClient, json: bool) -> Result<()> {
    if json {
        let raw = client.get_list_raw("/my/pins", true).await?;
        output::print_json(&raw);
    } else {
        let cards: Vec<Card> = client.get_list("/my/pins", true).await?;
        output::print_pins(&cards);
    }
    Ok(())
}
