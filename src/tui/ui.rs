use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::commands::agent::{deps_satisfied, parse_deps, unsatisfied_deps};
use crate::output::relative_time;

use super::app::{App, CardDetail, Modal, StatusKind, View};
use super::theme;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let board_area = chunks[0];
    let status_area = chunks[1];

    match app.view {
        View::Board => render_board(frame, app, board_area),
        View::CardDetail => {
            render_board(frame, app, board_area);
            if let Some(ref detail) = app.card_detail {
                render_card_detail_modal(frame, detail, area);
            }
        }
    }

    render_status_bar(frame, app, status_area);

    if let Some(ref modal) = app.modal {
        render_modal(frame, modal, area);
    }
}

fn render_board(frame: &mut Frame, app: &App, area: Rect) {
    if app.columns.is_empty() {
        let msg = if app.loading {
            "Loading..."
        } else {
            "No columns"
        };
        frame.render_widget(
            Paragraph::new(msg).style(theme::dim_meta()),
            area,
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    // Board title
    let board_name = app
        .board
        .as_ref()
        .map(|b| b.name.as_str())
        .unwrap_or("Board");
    let title_line = Line::from(vec![
        Span::styled(format!(" {board_name} "), theme::board_title()),
        Span::styled(
            format!(
                "  {} cards",
                app.cards.len()
            ),
            theme::dim_meta(),
        ),
    ]);
    frame.render_widget(Paragraph::new(title_line), chunks[0]);

    // Columns
    let num_cols = app.columns.len();
    let col_constraints: Vec<Constraint> = (0..num_cols)
        .map(|_| Constraint::Ratio(1, num_cols as u32))
        .collect();

    let col_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(col_constraints)
        .split(chunks[1]);

    for (i, col_area) in col_areas.iter().enumerate() {
        render_column(frame, app, i, *col_area);
    }
}

fn render_column(frame: &mut Frame, app: &App, col_idx: usize, area: Rect) {
    let col = &app.columns[col_idx];
    let is_selected = col_idx == app.selected_column;
    let cards = app.cards_in_column(col_idx);
    let card_count = cards.len();

    let col_color = theme::column_color(&col.color);
    let header_style = Style::default()
        .fg(col_color)
        .add_modifier(Modifier::BOLD);
    let border_style = if is_selected {
        theme::column_border_selected()
    } else {
        theme::column_border_normal()
    };

    let block = Block::default()
        .title(format!(" {} ({}) ", col.name, card_count))
        .title_style(header_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if cards.is_empty() {
        return;
    }

    let card_height: u16 = 3;
    let visible = (inner.height / card_height) as usize;

    // Ensure scroll keeps selected card visible
    let scroll = if is_selected {
        let scroll = app
            .scroll_offsets
            .get(col_idx)
            .copied()
            .unwrap_or(0);
        if app.selected_card >= scroll + visible {
            app.selected_card.saturating_sub(visible - 1)
        } else if app.selected_card < scroll {
            app.selected_card
        } else {
            scroll
        }
    } else {
        app.scroll_offsets.get(col_idx).copied().unwrap_or(0)
    };

    for (vis_idx, (card_idx, card)) in cards
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible)
        .enumerate()
    {
        let y = inner.y + (vis_idx as u16) * card_height;
        if y + card_height > inner.y + inner.height {
            break;
        }
        let card_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: card_height,
        };

        let is_card_selected = is_selected && card_idx == app.selected_card;
        render_card_compact(frame, card, card_area, is_card_selected, &app.cards);
    }

    // Scroll indicators
    if scroll > 0 {
        let indicator = Span::styled(" \u{25b2} ", theme::dim_meta());
        frame.render_widget(
            Paragraph::new(Line::from(indicator)),
            Rect {
                x: inner.x + inner.width.saturating_sub(3),
                y: inner.y,
                width: 3,
                height: 1,
            },
        );
    }
    if scroll + visible < cards.len() {
        let indicator = Span::styled(" \u{25bc} ", theme::dim_meta());
        let y = inner.y + inner.height.saturating_sub(1);
        frame.render_widget(
            Paragraph::new(Line::from(indicator)),
            Rect {
                x: inner.x + inner.width.saturating_sub(3),
                y,
                width: 3,
                height: 1,
            },
        );
    }
}

fn render_card_compact(
    frame: &mut Frame,
    card: &crate::models::Card,
    area: Rect,
    selected: bool,
    all_cards: &[crate::models::Card],
) {
    let width = area.width as usize;

    // Line 1: #N title
    let number_str = format!("#{}", card.number);
    let golden_marker = if card.golden { "\u{2605} " } else { "" };
    let title_max = width
        .saturating_sub(number_str.len() + 1 + golden_marker.len());
    let title = truncate(&card.title, title_max);

    let line1 = if card.golden && !selected {
        Line::from(vec![
            Span::styled(golden_marker, theme::golden_accent()),
            Span::styled(format!("{number_str} "), theme::dim_meta()),
            Span::raw(title),
        ])
    } else {
        Line::from(vec![
            Span::raw(golden_marker.to_string()),
            Span::styled(format!("{number_str} "), theme::dim_meta()),
            Span::raw(title),
        ])
    };

    // Line 2: @initials  tags  dep status
    let mut meta_parts: Vec<Span> = Vec::new();

    if let Some(ref assignees) = card.assignees {
        if !assignees.is_empty() {
            let initials: Vec<String> = assignees
                .iter()
                .take(2)
                .map(|u| user_initials(&u.name))
                .collect();
            meta_parts.push(Span::styled(
                format!("@{}", initials.join(",")),
                theme::dim_meta(),
            ));
        }
    }

    let user_tags: Vec<&String> = card
        .tags
        .iter()
        .filter(|t| !t.starts_with("after-"))
        .collect();
    if !user_tags.is_empty() {
        let tag_str: String = user_tags
            .iter()
            .take(2)
            .map(|t| format!("#{t}"))
            .collect::<Vec<_>>()
            .join(" ");
        meta_parts.push(Span::styled(format!(" {tag_str}"), theme::dim_meta()));
    }

    // Dependency indicator
    let deps = parse_deps(&card.tags);
    if !deps.is_empty() {
        if deps_satisfied(card, all_cards) {
            meta_parts.push(Span::styled(" \u{2713}", theme::ready_indicator()));
        } else {
            let blocked = unsatisfied_deps(card, all_cards);
            let blocked_str: Vec<String> = blocked.iter().map(|n| format!("#{n}")).collect();
            meta_parts.push(Span::styled(
                format!(" \u{2717}{}", blocked_str.join(",")),
                theme::blocked_indicator(),
            ));
        }
    }

    let line2 = Line::from(vec![Span::raw(" ")].into_iter().chain(meta_parts).collect::<Vec<_>>());

    // Line 3: separator
    let sep = "\u{2500}".repeat(width.min(60));
    let line3 = Line::from(Span::styled(sep, theme::column_border_normal()));

    let style = if selected {
        theme::selected_card()
    } else {
        Style::default()
    };

    let text = vec![line1, line2, line3];
    frame.render_widget(Paragraph::new(text).style(style), area);
}

fn render_card_detail_modal(frame: &mut Frame, detail: &CardDetail, area: Rect) {
    let modal_area = centered_rect(80, 85, area);
    frame.render_widget(Clear, modal_area);

    let card = &detail.card;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));

    // Metadata
    let col_name = card
        .column
        .as_ref()
        .map(|c| c.name.as_str())
        .unwrap_or("Triage");
    lines.push(Line::from(vec![
        Span::styled(" Column:   ", theme::dim_meta()),
        Span::raw(col_name),
    ]));

    if let Some(ref assignees) = card.assignees {
        if !assignees.is_empty() {
            let names: Vec<&str> = assignees.iter().map(|u| u.name.as_str()).collect();
            lines.push(Line::from(vec![
                Span::styled(" Assigned: ", theme::dim_meta()),
                Span::raw(names.join(", ")),
            ]));
        }
    }

    if !card.tags.is_empty() {
        let tag_str: String = card
            .tags
            .iter()
            .map(|t| format!("#{t}"))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(Line::from(vec![
            Span::styled(" Tags:     ", theme::dim_meta()),
            Span::raw(tag_str),
        ]));
    }

    if card.golden {
        lines.push(Line::from(vec![
            Span::styled(" Golden:   ", theme::dim_meta()),
            Span::styled("\u{2605} Yes", theme::golden_accent()),
        ]));
    }

    lines.push(Line::from(vec![
        Span::styled(" Created:  ", theme::dim_meta()),
        Span::raw(relative_time(&card.created_at)),
    ]));

    lines.push(Line::from(""));

    // Description
    if !card.description.is_empty() {
        lines.push(Line::from(Span::styled(
            " \u{2500}\u{2500}\u{2500} Description \u{2500}\u{2500}\u{2500}",
            theme::dim_meta(),
        )));
        for line in card.description.lines() {
            lines.push(Line::from(format!(" {line}")));
        }
        lines.push(Line::from(""));
    }

    // Steps
    if let Some(ref steps) = card.steps {
        if !steps.is_empty() {
            lines.push(Line::from(Span::styled(
                " \u{2500}\u{2500}\u{2500} Steps \u{2500}\u{2500}\u{2500}",
                theme::dim_meta(),
            )));
            for s in steps {
                let check = if s.completed { "[\u{2713}]" } else { "[ ]" };
                lines.push(Line::from(format!(" {check} {}", s.content)));
            }
            lines.push(Line::from(""));
        }
    }

    // Comments
    if !detail.comments.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(
                " \u{2500}\u{2500}\u{2500} Comments ({}) \u{2500}\u{2500}\u{2500}",
                detail.comments.len()
            ),
            theme::dim_meta(),
        )));
        for c in &detail.comments {
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", c.creator.name),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("\u{00b7} {}", relative_time(&c.created_at)),
                    theme::dim_meta(),
                ),
            ]));
            for line in c.body.plain_text.lines() {
                lines.push(Line::from(format!("   {line}")));
            }
            lines.push(Line::from(""));
        }
    }

    // Apply scroll
    let scroll = detail.scroll;
    let visible_lines: Vec<Line> = lines.into_iter().skip(scroll).collect();

    let golden_marker = if card.golden { "\u{2605} " } else { "" };
    let title = format!(" {golden_marker}#{} {} ", card.number, card.title);

    let block = Block::default()
        .title(title)
        .title_style(theme::board_title())
        .borders(Borders::ALL)
        .border_style(theme::column_border_selected());

    let footer = Line::from(vec![
        Span::styled(" Esc", theme::help_key()),
        Span::styled(":back ", theme::help_desc()),
        Span::styled("m", theme::help_key()),
        Span::styled(":move ", theme::help_desc()),
        Span::styled("a", theme::help_key()),
        Span::styled(":assign ", theme::help_desc()),
        Span::styled("c", theme::help_key()),
        Span::styled(":comment ", theme::help_desc()),
        Span::styled("g", theme::help_key()),
        Span::styled(":gold ", theme::help_desc()),
        Span::styled("j/k", theme::help_key()),
        Span::styled(":scroll", theme::help_desc()),
    ]);

    let block = block
        .title_bottom(footer)
        .title_alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(
        Paragraph::new(visible_lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        modal_area,
    );
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    // Left: status message or keybindings
    if let Some((ref msg, ref kind)) = app.status_message {
        let style = match kind {
            StatusKind::Info => theme::status_info(),
            StatusKind::Success => theme::status_success(),
            StatusKind::Error => theme::status_error(),
        };
        spans.push(Span::styled(format!(" {msg}"), style));
    } else {
        spans.push(Span::styled(" h/l", theme::help_key()));
        spans.push(Span::styled(":col ", theme::help_desc()));
        spans.push(Span::styled("j/k", theme::help_key()));
        spans.push(Span::styled(":card ", theme::help_desc()));
        spans.push(Span::styled("Enter", theme::help_key()));
        spans.push(Span::styled(":open ", theme::help_desc()));
        spans.push(Span::styled("m", theme::help_key()));
        spans.push(Span::styled(":move ", theme::help_desc()));
        spans.push(Span::styled("a", theme::help_key()));
        spans.push(Span::styled(":assign ", theme::help_desc()));
        spans.push(Span::styled("c", theme::help_key()));
        spans.push(Span::styled(":comment ", theme::help_desc()));
        spans.push(Span::styled("g", theme::help_key()));
        spans.push(Span::styled(":gold ", theme::help_desc()));
        spans.push(Span::styled("r", theme::help_key()));
        spans.push(Span::styled(":refresh ", theme::help_desc()));
        spans.push(Span::styled("?", theme::help_key()));
        spans.push(Span::styled(":help ", theme::help_desc()));
        spans.push(Span::styled("q", theme::help_key()));
        spans.push(Span::styled(":quit", theme::help_desc()));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_modal(frame: &mut Frame, modal: &Modal, area: Rect) {
    match modal {
        Modal::ColumnPicker { options, selected } => {
            let modal_area = centered_rect(40, 50, area);
            frame.render_widget(Clear, modal_area);

            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(""));
            for (i, col) in options.iter().enumerate() {
                let marker = if i == *selected { " \u{25b6} " } else { "   " };
                let style = if i == *selected {
                    theme::selected_card()
                } else {
                    Style::default()
                };
                lines.push(Line::from(Span::styled(
                    format!("{marker}{}", col.name),
                    style,
                )));
            }

            let block = Block::default()
                .title(" Move to column ")
                .title_style(theme::board_title())
                .borders(Borders::ALL)
                .border_style(theme::column_border_selected());

            frame.render_widget(Paragraph::new(lines).block(block), modal_area);
        }
        Modal::AssignPicker { options, selected } => {
            let modal_area = centered_rect(40, 50, area);
            frame.render_widget(Clear, modal_area);

            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(""));
            for (i, user) in options.iter().enumerate() {
                let marker = if i == *selected { " \u{25b6} " } else { "   " };
                let style = if i == *selected {
                    theme::selected_card()
                } else {
                    Style::default()
                };
                lines.push(Line::from(Span::styled(
                    format!("{marker}{}", user.name),
                    style,
                )));
            }

            let block = Block::default()
                .title(" Assign to user ")
                .title_style(theme::board_title())
                .borders(Borders::ALL)
                .border_style(theme::column_border_selected());

            frame.render_widget(Paragraph::new(lines).block(block), modal_area);
        }
        Modal::CommentInput { buffer } => {
            let modal_area = centered_rect(60, 20, area);
            frame.render_widget(Clear, modal_area);

            let lines = vec![
                Line::from(""),
                Line::from(format!(" > {buffer}\u{2588}")),
            ];

            let block = Block::default()
                .title(" Add comment (Enter to submit, Esc to cancel) ")
                .title_style(theme::board_title())
                .borders(Borders::ALL)
                .border_style(theme::column_border_selected());

            frame.render_widget(Paragraph::new(lines).block(block), modal_area);
        }
        Modal::Help => {
            let modal_area = centered_rect(50, 70, area);
            frame.render_widget(Clear, modal_area);

            let lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Navigation", Style::default().add_modifier(Modifier::BOLD)),
                ]),
                help_line("h/\u{2190}", "Previous column"),
                help_line("l/\u{2192}", "Next column"),
                help_line("k/\u{2191}", "Previous card"),
                help_line("j/\u{2193}", "Next card"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Actions", Style::default().add_modifier(Modifier::BOLD)),
                ]),
                help_line("Enter", "Open card detail"),
                help_line("Esc", "Go back / close"),
                help_line("m", "Move card to column"),
                help_line("a", "Assign / unassign user"),
                help_line("c", "Add comment"),
                help_line("g", "Toggle golden"),
                help_line("r", "Refresh data"),
                help_line("?", "This help"),
                help_line("q", "Quit"),
            ];

            let block = Block::default()
                .title(" Help ")
                .title_style(theme::board_title())
                .borders(Borders::ALL)
                .border_style(theme::column_border_selected());

            frame.render_widget(Paragraph::new(lines).block(block), modal_area);
        }
    }
}

// --- Helpers ---

fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{key:>8}"), theme::help_key()),
        Span::raw("  "),
        Span::styled(desc, theme::help_desc()),
    ])
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 1 {
        format!("{}\u{2026}", &s[..max - 1])
    } else {
        "\u{2026}".to_string()
    }
}

fn user_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase()
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
