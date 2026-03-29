use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::models::{Board, Card, Column, Comment, User};

pub enum AppEvent {
    Terminal(CrosstermEvent),
    ApiResult(ApiResult),
    Tick,
}

pub enum ApiResult {
    BoardLoaded {
        board: Board,
        columns: Vec<Column>,
        cards: Vec<Card>,
        users: Vec<User>,
        my_id: String,
        my_name: String,
    },
    CardDetailLoaded {
        card: Card,
        comments: Vec<Comment>,
    },
    MutationDone {
        message: String,
    },
    Refreshed {
        columns: Vec<Column>,
        cards: Vec<Card>,
    },
    Error(String),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> (Self, mpsc::UnboundedSender<AppEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();

        let tx_term = tx.clone();
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            while let Some(Ok(event)) = reader.next().await {
                if tx_term.send(AppEvent::Terminal(event)).is_err() {
                    break;
                }
            }
        });

        let tx_tick = tx.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(tick_rate_ms));
            loop {
                interval.tick().await;
                if tx_tick.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
        });

        (Self { rx }, tx)
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
