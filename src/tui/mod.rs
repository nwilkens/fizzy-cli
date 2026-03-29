mod app;
mod event;
mod theme;
mod ui;

use crate::client::FizzyClient;
use event::{ApiResult, AppEvent};

pub async fn run_tui(client: FizzyClient, board_id: &str) -> anyhow::Result<()> {
    // Enter alternate screen + raw mode
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Event handler
    let (mut events, api_tx) = event::EventHandler::new(250);

    // App state (loading)
    let mut app_state = app::App::new_loading();

    // Kick off initial data load
    let load_client = client.clone();
    let load_board_id = board_id.to_string();
    let load_tx = api_tx.clone();
    tokio::spawn(async move {
        match app::load_board_full(&load_client, &load_board_id).await {
            Ok((board, columns, cards, users, my_id, my_name)) => {
                let _ = load_tx.send(AppEvent::ApiResult(ApiResult::BoardLoaded {
                    board,
                    columns,
                    cards,
                    users,
                    my_id,
                    my_name,
                }));
            }
            Err(e) => {
                let _ = load_tx.send(AppEvent::ApiResult(ApiResult::Error(format!(
                    "Failed to load board: {e}"
                ))));
            }
        }
    });

    // Main loop
    let result = run_loop(&mut terminal, &mut app_state, &mut events, &client, &api_tx).await;

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut app::App,
    events: &mut event::EventHandler,
    client: &FizzyClient,
    tx: &tokio::sync::mpsc::UnboundedSender<AppEvent>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if let Some(event) = events.next().await {
            app.handle_event(event, client, tx);
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
