use crate::display::App;
use crate::settings::Branch;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, List, ListItem, ListState};
use tui::Frame;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    if app.notifs.enabled {
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(app.visible_notifs + 2),
                    Constraint::Min(0),
                    Constraint::Length(7),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());

        draw_notifs(f, app, chunks[0]);
        draw_repos(f, app, chunks[1]);
        draw_recent(f, app, chunks[2]);
        draw_filter(f, app, chunks[3]);
    } else {
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Min(0),
                    Constraint::Length(7),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());

        draw_repos(f, app, chunks[0]);
        draw_recent(f, app, chunks[1]);
        draw_filter(f, app, chunks[2]);
    }
}

fn draw_filter<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let style = match app.state.state.selected() {
        Some(2) => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        _ => match !app.filter.is_empty() {
            true => Style::default().fg(Color::Yellow),
            false => Style::default().fg(Color::White),
        },
    };
    let filter = ListItem::new(Span::styled(&app.filter, style));
    let rows = List::new([filter]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title(" Filter "),
    );
    f.render_widget(rows, area);
}

fn draw_notifs<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let style = Style::default().fg(Color::White);
    let notifs = app
        .notifs
        .all
        .items
        .iter()
        .filter(|(_, (_, visible))| *visible)
        .map(|(status, _)| {
            if !status.reason.is_empty() {
                format!(
                    "[{}] {} ({})",
                    status.repository.full_name, status.subject.title, status.reason
                )
            } else {
                format!("[{}] {}", status.repository.full_name, status.subject.title)
            }
        })
        .map(|text| ListItem::new(Span::styled(text, style)))
        .collect::<Vec<_>>();

    let current = &app
        .notifs
        .all
        .items
        .iter()
        .filter(|(_, (_, visible))| *visible)
        .count();
    let total = &app.notifs.all.items.len();
    let title = match current == total {
        true => format!(" Notifications ({}) ", &total),
        false => format!(" Notifications ({}/{}) ", &current, &total),
    };

    let style = match app.state.state.selected() {
        Some(0) => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::White),
    };
    let rows = List::new(notifs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(style)
                .title(Span::styled(&title, Style::default())),
        )
        .highlight_style(style);

    f.render_stateful_widget(rows, area, &mut app.notifs.all.state);
}

fn draw_repos<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let style_error = Style::default().fg(Color::Magenta);
    let style_failure = Style::default().fg(Color::Red);
    let style_success = Style::default().fg(Color::Green);
    let style_unknown = Style::default().fg(Color::White);
    let repos = app
        .repos
        .all
        .items
        .iter()
        .filter(|(_, (_, visible))| *visible)
        .map(|(repo, (status, _))| {
            let text = match app
                .repos
                .all
                .items
                .keys()
                .filter(|&r| r.name == repo.name)
                .count()
            {
                1 => format!("{}", repo.name),
                _ if repo.cctray.is_some() => format!("{}", repo.name),
                _ if repo.circleci.is_some() => {
                    let circleci = repo.circleci.as_ref().unwrap();
                    if circleci.branch == Branch::default() {
                        format!("{} ({})", repo.name, circleci.workflow)
                    } else {
                        format!(
                            "{} ({} on {})",
                            repo.name, circleci.workflow, circleci.branch
                        )
                    }
                }
                _ => format!("{}", repo.name), // impossible
            };
            (text, status)
        })
        .map(|(text, status)| {
            ListItem::new(Span::styled(
                text,
                match status.status.as_ref() {
                    "error" => style_error,
                    "failed" => style_failure,
                    "success" => style_success,
                    _ => style_unknown,
                },
            ))
        })
        .collect::<Vec<_>>();

    // TODO: better handling for too many repos to fit nicely on screen
    let height = area.height - 2; // header/footer
    let columns = ((app.repos.all.items.len() as u16) + height - 1) / height;
    let constraints = vec![Constraint::Percentage(100 / columns); columns as usize];
    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Horizontal)
        .split(area);

    for (i, (&chunk, rows)) in chunks.iter().zip(repos.chunks(height as usize)).enumerate() {
        let style = match app.state.state.selected() {
            Some(1) => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::White),
        };
        let rows = List::new(rows)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(style)
                    .title(Span::styled(" Repo Status ", Style::default())),
            )
            .highlight_style(style);

        // share our state selector across columns
        let mut local_state: &mut ListState = &mut (&*app.repos.all.state).clone();
        let selected = match local_state.selected() {
            Some(x) if x < (height as usize) * i => None,
            Some(x) if x >= (height as usize) * (i + 1) => None,
            Some(x) => Some(x - (height as usize * i)),
            None => None,
        };
        local_state.select(selected);
        f.render_stateful_widget(rows, chunk, &mut local_state);
    }
}

fn draw_recent<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let style_error = Style::default().fg(Color::Magenta);
    let style_failure = Style::default().fg(Color::Red);
    let style_success = Style::default().fg(Color::Green);
    let style_unknown = Style::default().fg(Color::White);
    let repos = app
        .repos
        .recent
        .items
        .iter()
        .rev()
        .filter(|(_, (_, visible))| *visible)
        .map(|(status, (repo, _))| {
            let text = if let Some(_) = &repo.cctray {
                format!("{}", repo.name)
            } else if let Some(circleci) = &repo.circleci {
                format!(
                    "{} ({} on {})",
                    repo.name, circleci.workflow, circleci.branch
                )
            } else {
                format!("{}", repo.name)
            };
            (text, status)
        })
        .map(|(text, status)| {
            ListItem::new(Span::styled(
                text,
                match status.status.as_ref() {
                    "error" => style_error,
                    "failed" => style_failure,
                    "success" => style_success,
                    _ => style_unknown,
                },
            ))
        })
        .collect::<Vec<_>>();

    let rows = List::new(repos).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Recent Workflows "),
    );

    f.render_stateful_widget(rows, area, &mut app.repos.recent.state);
}
