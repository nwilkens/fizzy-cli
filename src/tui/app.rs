use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

use crate::client::FizzyClient;
use crate::models::*;

use super::event::{ApiResult, AppEvent};

pub struct App {
    pub board: Option<Board>,
    pub columns: Vec<Column>,
    pub cards: Vec<Card>,
    pub users: Vec<User>,
    pub my_user_id: String,
    pub my_user_name: String,

    // Navigation
    pub selected_column: usize,
    pub selected_card: usize,
    pub scroll_offsets: Vec<usize>,

    // View state
    pub view: View,
    pub card_detail: Option<CardDetail>,

    // Modal
    pub modal: Option<Modal>,

    // Status
    pub status_message: Option<(String, StatusKind)>,
    pub status_ticks: u32,
    pub loading: bool,
    pub should_quit: bool,
}

#[derive(PartialEq)]
pub enum View {
    Board,
    CardDetail,
}

pub struct CardDetail {
    pub card: Card,
    pub comments: Vec<Comment>,
    pub scroll: usize,
}

pub enum Modal {
    ColumnPicker {
        options: Vec<Column>,
        selected: usize,
    },
    AssignPicker {
        options: Vec<User>,
        selected: usize,
    },
    CommentInput {
        buffer: String,
    },
    Help,
}

#[derive(Clone)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}

impl App {
    pub fn new_loading() -> Self {
        Self {
            board: None,
            columns: Vec::new(),
            cards: Vec::new(),
            users: Vec::new(),
            my_user_id: String::new(),
            my_user_name: String::new(),
            selected_column: 0,
            selected_card: 0,
            scroll_offsets: Vec::new(),
            view: View::Board,
            card_detail: None,
            modal: None,
            status_message: Some(("Loading board...".into(), StatusKind::Info)),
            status_ticks: 0,
            loading: true,
            should_quit: false,
        }
    }

    pub fn cards_in_column(&self, col_index: usize) -> Vec<&Card> {
        if col_index >= self.columns.len() {
            return Vec::new();
        }
        let col_id = &self.columns[col_index].id;
        self.cards
            .iter()
            .filter(|c| {
                c.column
                    .as_ref()
                    .map(|col| &col.id == col_id)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn selected_card_ref(&self) -> Option<&Card> {
        let cards = self.cards_in_column(self.selected_column);
        cards.get(self.selected_card).copied()
    }

    pub fn handle_event(
        &mut self,
        event: AppEvent,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        match event {
            AppEvent::Terminal(crossterm::event::Event::Key(key)) => {
                self.handle_key(key, client, tx);
            }
            AppEvent::Terminal(crossterm::event::Event::Resize(_, _)) => {}
            AppEvent::ApiResult(result) => {
                self.handle_api_result(result);
            }
            AppEvent::Tick => {
                self.tick_status();
            }
            _ => {}
        }
    }

    fn handle_key(
        &mut self,
        key: KeyEvent,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        if self.modal.is_some() {
            self.handle_modal_key(key, client, tx);
            return;
        }

        match self.view {
            View::Board => self.handle_board_key(key, client, tx),
            View::CardDetail => self.handle_detail_key(key, client, tx),
        }
    }

    fn handle_board_key(
        &mut self,
        key: KeyEvent,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => self.move_column(-1),
            KeyCode::Right | KeyCode::Char('l') => self.move_column(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_card(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_card(1),
            KeyCode::Enter => self.open_card_detail(client, tx),
            KeyCode::Char('m') => self.open_column_picker(),
            KeyCode::Char('a') => self.open_assign_picker(),
            KeyCode::Char('c') => self.open_comment_input(),
            KeyCode::Char('g') => self.toggle_golden(client, tx),
            KeyCode::Char('r') => self.refresh(client, tx),
            KeyCode::Char('?') => self.modal = Some(Modal::Help),
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_detail_key(
        &mut self,
        key: KeyEvent,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.view = View::Board;
                self.card_detail = None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(ref mut detail) = self.card_detail {
                    detail.scroll = detail.scroll.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(ref mut detail) = self.card_detail {
                    detail.scroll += 1;
                }
            }
            KeyCode::Char('m') => self.open_column_picker(),
            KeyCode::Char('a') => self.open_assign_picker(),
            KeyCode::Char('c') => self.open_comment_input(),
            KeyCode::Char('g') => self.toggle_golden(client, tx),
            KeyCode::Char('?') => self.modal = Some(Modal::Help),
            _ => {}
        }
    }

    fn handle_modal_key(
        &mut self,
        key: KeyEvent,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        let modal = self.modal.take();
        match modal {
            Some(Modal::ColumnPicker {
                options,
                mut selected,
            }) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.saturating_sub(1);
                    self.modal = Some(Modal::ColumnPicker { options, selected });
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected + 1 < options.len() {
                        selected += 1;
                    }
                    self.modal = Some(Modal::ColumnPicker { options, selected });
                }
                KeyCode::Enter => {
                    let target = &options[selected];
                    self.execute_move_card(target.id.clone(), target.name.clone(), client, tx);
                }
                KeyCode::Esc => {}
                _ => {
                    self.modal = Some(Modal::ColumnPicker { options, selected });
                }
            },
            Some(Modal::AssignPicker {
                options,
                mut selected,
            }) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.saturating_sub(1);
                    self.modal = Some(Modal::AssignPicker { options, selected });
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected + 1 < options.len() {
                        selected += 1;
                    }
                    self.modal = Some(Modal::AssignPicker { options, selected });
                }
                KeyCode::Enter => {
                    let target = &options[selected];
                    self.execute_assign(target.id.clone(), target.name.clone(), client, tx);
                }
                KeyCode::Esc => {}
                _ => {
                    self.modal = Some(Modal::AssignPicker { options, selected });
                }
            },
            Some(Modal::CommentInput { mut buffer }) => match key.code {
                KeyCode::Enter => {
                    if !buffer.is_empty() {
                        self.execute_add_comment(buffer, client, tx);
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Char(c) => {
                    buffer.push(c);
                    self.modal = Some(Modal::CommentInput { buffer });
                }
                KeyCode::Backspace => {
                    buffer.pop();
                    self.modal = Some(Modal::CommentInput { buffer });
                }
                _ => {
                    self.modal = Some(Modal::CommentInput { buffer });
                }
            },
            Some(Modal::Help) => match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {}
                _ => {
                    self.modal = Some(Modal::Help);
                }
            },
            None => {}
        }
    }

    fn handle_api_result(&mut self, result: ApiResult) {
        match result {
            ApiResult::BoardLoaded {
                board,
                columns,
                cards,
                users,
                my_id,
                my_name,
            } => {
                self.board = Some(board);
                self.columns = columns;
                self.cards = cards;
                self.users = users;
                self.my_user_id = my_id;
                self.my_user_name = my_name;
                self.scroll_offsets = vec![0; self.columns.len()];
                self.loading = false;
                self.set_status("Board loaded", StatusKind::Success);
            }
            ApiResult::CardDetailLoaded { card, comments } => {
                self.card_detail = Some(CardDetail {
                    card,
                    comments,
                    scroll: 0,
                });
                self.view = View::CardDetail;
            }
            ApiResult::MutationDone { message } => {
                self.set_status(&message, StatusKind::Success);
            }
            ApiResult::Refreshed { columns, cards } => {
                self.columns = columns;
                self.cards = cards;
                self.scroll_offsets
                    .resize(self.columns.len(), 0);
                self.loading = false;
                self.set_status("Refreshed", StatusKind::Success);
            }
            ApiResult::Error(msg) => {
                self.loading = false;
                self.set_status(&msg, StatusKind::Error);
            }
        }
    }

    // --- Navigation ---

    fn move_column(&mut self, delta: i32) {
        if self.columns.is_empty() {
            return;
        }
        let new = self.selected_column as i32 + delta;
        if new >= 0 && (new as usize) < self.columns.len() {
            self.selected_column = new as usize;
            let count = self.cards_in_column(self.selected_column).len();
            if self.selected_card >= count {
                self.selected_card = count.saturating_sub(1);
            }
        }
    }

    fn move_card(&mut self, delta: i32) {
        let count = self.cards_in_column(self.selected_column).len();
        if count == 0 {
            return;
        }
        let new = self.selected_card as i32 + delta;
        if new >= 0 && (new as usize) < count {
            self.selected_card = new as usize;
            self.adjust_scroll();
        }
    }

    fn adjust_scroll(&mut self) {
        if self.selected_column >= self.scroll_offsets.len() {
            return;
        }
        let scroll = self.scroll_offsets[self.selected_column];
        if self.selected_card < scroll {
            self.scroll_offsets[self.selected_column] = self.selected_card;
        }
    }

    // --- Actions ---

    fn open_card_detail(
        &mut self,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        let card = self.selected_card_ref();
        if card.is_none() {
            return;
        }
        let card_number = card.unwrap().number;
        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            match load_card_detail(&client, card_number).await {
                Ok((card, comments)) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::CardDetailLoaded {
                        card,
                        comments,
                    }));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::Error(format!(
                        "Load failed: {e}"
                    ))));
                }
            }
        });
    }

    fn open_column_picker(&mut self) {
        if self.selected_card_ref().is_none() {
            return;
        }
        self.modal = Some(Modal::ColumnPicker {
            options: self.columns.clone(),
            selected: 0,
        });
    }

    fn open_assign_picker(&mut self) {
        if self.selected_card_ref().is_none() {
            return;
        }
        if self.users.is_empty() {
            self.set_status("No users loaded", StatusKind::Error);
            return;
        }
        self.modal = Some(Modal::AssignPicker {
            options: self.users.clone(),
            selected: 0,
        });
    }

    fn open_comment_input(&mut self) {
        if self.selected_card_ref().is_none() {
            return;
        }
        self.modal = Some(Modal::CommentInput {
            buffer: String::new(),
        });
    }

    fn toggle_golden(
        &mut self,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        let card = self.selected_card_ref();
        if card.is_none() {
            return;
        }
        let card_number = card.unwrap().number;
        let was_golden = card.unwrap().golden;

        // Optimistic update
        if let Some(c) = self.cards.iter_mut().find(|c| c.number == card_number) {
            c.golden = !was_golden;
        }

        let client = client.clone();
        let tx = tx.clone();
        let endpoint = format!("/cards/{card_number}/goldness");
        tokio::spawn(async move {
            let result = if was_golden {
                client.delete(&endpoint).await
            } else {
                client.post_no_body(&endpoint).await
            };
            match result {
                Ok(_) => {
                    let status = if was_golden { "unmarked" } else { "marked" };
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::MutationDone {
                        message: format!("#{card_number} {status} golden"),
                    }));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::Error(format!(
                        "Golden toggle failed: {e}"
                    ))));
                }
            }
        });
    }

    fn execute_move_card(
        &mut self,
        column_id: String,
        column_name: String,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        let card = self.selected_card_ref();
        if card.is_none() {
            return;
        }
        let card_number = card.unwrap().number;

        // Optimistic: update column
        if let Some(c) = self.cards.iter_mut().find(|c| c.number == card_number) {
            let target_col = self.columns.iter().find(|col| col.id == column_id).cloned();
            c.column = target_col;
        }

        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let body = TriageRequest {
                column_id: column_id.clone(),
            };
            match client
                .post_raw(&format!("/cards/{card_number}/triage"), &body)
                .await
            {
                Ok(_) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::MutationDone {
                        message: format!("#{card_number} moved to {column_name}"),
                    }));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::Error(format!(
                        "Move failed: {e}"
                    ))));
                }
            }
        });
    }

    fn execute_assign(
        &mut self,
        user_id: String,
        user_name: String,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        let card = self.selected_card_ref();
        if card.is_none() {
            return;
        }
        let card_number = card.unwrap().number;

        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let body = AssignmentRequest {
                assignee_id: user_id,
            };
            match client
                .post_raw(&format!("/cards/{card_number}/assignments"), &body)
                .await
            {
                Ok(_) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::MutationDone {
                        message: format!("#{card_number} assignment toggled for {user_name}"),
                    }));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::Error(format!(
                        "Assign failed: {e}"
                    ))));
                }
            }
        });
    }

    fn execute_add_comment(
        &mut self,
        body_text: String,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        let card = self.selected_card_ref();
        if card.is_none() {
            return;
        }
        let card_number = card.unwrap().number;

        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let body = CreateCommentRequest {
                comment: CreateCommentBody {
                    body: body_text,
                },
            };
            match client
                .post_raw(&format!("/cards/{card_number}/comments"), &body)
                .await
            {
                Ok(_) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::MutationDone {
                        message: format!("Comment added to #{card_number}"),
                    }));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::Error(format!(
                        "Comment failed: {e}"
                    ))));
                }
            }
        });
    }

    pub fn refresh(
        &mut self,
        client: &FizzyClient,
        tx: &mpsc::UnboundedSender<AppEvent>,
    ) {
        if self.loading {
            return;
        }
        self.loading = true;
        self.set_status("Refreshing...", StatusKind::Info);

        let board_id = match &self.board {
            Some(b) => b.id.clone(),
            None => return,
        };

        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            match load_board_data_partial(&client, &board_id).await {
                Ok((columns, cards)) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::Refreshed { columns, cards }));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ApiResult(ApiResult::Error(format!(
                        "Refresh failed: {e}"
                    ))));
                }
            }
        });
    }

    // --- Status ---

    fn set_status(&mut self, msg: &str, kind: StatusKind) {
        self.status_message = Some((msg.to_string(), kind));
        self.status_ticks = 0;
    }

    fn tick_status(&mut self) {
        if self.status_message.is_some() {
            self.status_ticks += 1;
            // Clear after ~3 seconds (12 ticks at 250ms)
            if self.status_ticks > 12 {
                self.status_message = None;
                self.status_ticks = 0;
            }
        }
    }
}

// --- Async data loading ---

pub async fn load_board_full(
    client: &FizzyClient,
    board_id: &str,
) -> anyhow::Result<(Board, Vec<Column>, Vec<Card>, Vec<User>, String, String)> {
    let board: Board = client.get(&format!("/boards/{board_id}")).await?;
    let columns: Vec<Column> = client
        .get_list(&format!("/boards/{board_id}/columns"), true)
        .await?;
    let cards: Vec<Card> = client
        .get_list(&format!("/cards?board_ids[]={board_id}"), true)
        .await?;
    let users: Vec<User> = client.get_list("/users", true).await?;

    let identity: IdentityResponse = client.get_global("/my/identity").await?;
    let slug = client.account_slug();
    let account = identity
        .accounts
        .iter()
        .find(|a| a.slug.trim_start_matches('/') == slug)
        .ok_or_else(|| anyhow::anyhow!("No user for account /{slug}"))?;
    let my_id = account.user.id.clone();
    let my_name = account.user.name.clone();

    Ok((board, columns, cards, users, my_id, my_name))
}

async fn load_board_data_partial(
    client: &FizzyClient,
    board_id: &str,
) -> anyhow::Result<(Vec<Column>, Vec<Card>)> {
    let columns: Vec<Column> = client
        .get_list(&format!("/boards/{board_id}/columns"), true)
        .await?;
    let cards: Vec<Card> = client
        .get_list(&format!("/cards?board_ids[]={board_id}"), true)
        .await?;
    Ok((columns, cards))
}

async fn load_card_detail(
    client: &FizzyClient,
    card_number: u64,
) -> anyhow::Result<(Card, Vec<Comment>)> {
    let card: Card = client.get(&format!("/cards/{card_number}")).await?;
    let comments: Vec<Comment> = client
        .get_list(&format!("/cards/{card_number}/comments"), true)
        .await?;
    Ok((card, comments))
}
