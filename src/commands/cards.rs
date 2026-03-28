use anyhow::Result;
use crate::client::FizzyClient;
use crate::models::*;
use crate::output;

pub async fn list(
    client: &FizzyClient,
    board: Option<&str>,
    column: Option<&str>,
    assignee: Option<&str>,
    tag: Option<&str>,
    index: Option<&str>,
    sort: Option<&str>,
    search: Option<&str>,
    fetch_all: bool,
    json: bool,
) -> Result<()> {
    let mut params = Vec::new();
    if let Some(b) = board {
        params.push(format!("board_ids[]={b}"));
    }
    if let Some(c) = column {
        // column filtering is done client-side since API only has board_ids
        // Actually, the API doesn't have column_id filter on cards index
        // We'll filter after fetching if needed
        let _ = c; // placeholder
    }
    if let Some(a) = assignee {
        params.push(format!("assignee_ids[]={a}"));
    }
    if let Some(t) = tag {
        params.push(format!("tag_ids[]={t}"));
    }
    if let Some(i) = index {
        params.push(format!("indexed_by={i}"));
    }
    if let Some(s) = sort {
        params.push(format!("sorted_by={s}"));
    }
    if let Some(s) = search {
        for term in s.split_whitespace() {
            params.push(format!("terms[]={term}"));
        }
    }

    let query = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    let path = format!("/cards{query}");

    if json {
        let raw = client.get_list_raw(&path, fetch_all).await?;
        output::print_json(&raw);
    } else {
        let cards: Vec<Card> = client.get_list(&path, fetch_all).await?;
        output::print_cards(&cards);
    }
    Ok(())
}

pub async fn show(client: &FizzyClient, number: u64, json: bool) -> Result<()> {
    if json {
        let raw = client.get_raw(&format!("/cards/{number}")).await?;
        output::print_json(&raw);
    } else {
        let card: Card = client.get(&format!("/cards/{number}")).await?;
        output::print_card_detail(&card);
    }
    Ok(())
}

pub async fn create(
    client: &FizzyClient,
    board_id: &str,
    title: &str,
    description: Option<&str>,
    tags: Option<&str>,
    draft: bool,
    json: bool,
) -> Result<()> {
    let tag_ids = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
    let body = CreateCardRequest {
        card: CreateCardBody {
            title: title.to_string(),
            description: description.map(|d| format!("<p>{d}</p>")),
            status: if draft { Some("drafted".to_string()) } else { None },
            tag_ids,
        },
    };
    let raw = client.post_raw(&format!("/boards/{board_id}/cards"), &body).await?;
    if json {
        output::print_json(&raw);
    } else {
        println!("Card created.");
    }
    Ok(())
}

pub async fn update(
    client: &FizzyClient,
    number: u64,
    title: Option<String>,
    description: Option<String>,
    _json: bool,
) -> Result<()> {
    let body = UpdateCardRequest {
        card: UpdateCardBody {
            title,
            description: description.map(|d| format!("<p>{d}</p>")),
            status: None,
            tag_ids: None,
        },
    };
    client.put(&format!("/cards/{number}"), &body).await?;
    println!("Card updated.");
    Ok(())
}

pub async fn delete(client: &FizzyClient, number: u64) -> Result<()> {
    client.delete(&format!("/cards/{number}")).await?;
    println!("Card deleted.");
    Ok(())
}

pub async fn close(client: &FizzyClient, number: u64) -> Result<()> {
    client.post_no_body(&format!("/cards/{number}/closure")).await?;
    println!("Card #{number} closed.");
    Ok(())
}

pub async fn reopen(client: &FizzyClient, number: u64) -> Result<()> {
    client.delete(&format!("/cards/{number}/closure")).await?;
    println!("Card #{number} reopened.");
    Ok(())
}

pub async fn postpone(client: &FizzyClient, number: u64) -> Result<()> {
    client.post_no_body(&format!("/cards/{number}/not_now")).await?;
    println!("Card #{number} postponed.");
    Ok(())
}

pub async fn triage(client: &FizzyClient, number: u64, column_id: &str) -> Result<()> {
    let body = TriageRequest {
        column_id: column_id.to_string(),
    };
    client.post_raw(&format!("/cards/{number}/triage"), &body).await?;
    println!("Card #{number} triaged.");
    Ok(())
}

pub async fn untriage(client: &FizzyClient, number: u64) -> Result<()> {
    client.delete(&format!("/cards/{number}/triage")).await?;
    println!("Card #{number} sent back to triage.");
    Ok(())
}

pub async fn tag(client: &FizzyClient, number: u64, tag_title: &str) -> Result<()> {
    let body = TaggingRequest {
        tag_title: tag_title.to_string(),
    };
    client.post_raw(&format!("/cards/{number}/taggings"), &body).await?;
    println!("Tag toggled on card #{number}.");
    Ok(())
}

pub async fn assign(client: &FizzyClient, number: u64, assignee_id: &str) -> Result<()> {
    let body = AssignmentRequest {
        assignee_id: assignee_id.to_string(),
    };
    client.post_raw(&format!("/cards/{number}/assignments"), &body).await?;
    println!("Assignment toggled on card #{number}.");
    Ok(())
}

pub async fn watch(client: &FizzyClient, number: u64) -> Result<()> {
    client.post_no_body(&format!("/cards/{number}/watch")).await?;
    println!("Watching card #{number}.");
    Ok(())
}

pub async fn unwatch(client: &FizzyClient, number: u64) -> Result<()> {
    client.delete(&format!("/cards/{number}/watch")).await?;
    println!("Unwatched card #{number}.");
    Ok(())
}

pub async fn gold(client: &FizzyClient, number: u64) -> Result<()> {
    client.post_no_body(&format!("/cards/{number}/goldness")).await?;
    println!("Card #{number} marked golden.");
    Ok(())
}

pub async fn ungold(client: &FizzyClient, number: u64) -> Result<()> {
    client.delete(&format!("/cards/{number}/goldness")).await?;
    println!("Card #{number} golden status removed.");
    Ok(())
}

pub async fn pin(client: &FizzyClient, number: u64) -> Result<()> {
    client.post_no_body(&format!("/cards/{number}/pin")).await?;
    println!("Card #{number} pinned.");
    Ok(())
}

pub async fn unpin(client: &FizzyClient, number: u64) -> Result<()> {
    client.delete(&format!("/cards/{number}/pin")).await?;
    println!("Card #{number} unpinned.");
    Ok(())
}
