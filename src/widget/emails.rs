use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Margin, Position, Rect},
    style::{Color, Style, Stylize},
    symbols::line,
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
    },
    Frame,
};
use unicode_width::UnicodeWidthChar as _;

use crate::app::{Context, LoadType, Mode};

use super::{Focusable, StatefulTable, Widget};

pub struct EmailEntry {
    pub from: String,
    pub subject: String,
    pub date: String,
}

pub struct Emails {
    pub table: StatefulTable<EmailEntry>,
    last_area: Rect,
}

impl Default for Emails {
    fn default() -> Self {
        Emails {
            table: StatefulTable::new(),
            last_area: Rect::default(),
        }
    }
}

impl Emails {
    pub fn set_entries(&mut self, entries: Vec<EmailEntry>) {
        self.table.with_items(entries);
    }
}

impl Widget for Emails {
    fn draw(&mut self, f: &mut Frame, area: Rect, ctx: &mut Context) {
        let title = match ctx.mode == Mode::Loading(LoadType::FetchEmails) {
            true => "Loading...".to_owned(),
            false => format!("Emails: {}", ctx.config.username),
        };
        // let rows: Vec<Row> = vec![];
        let rows = self.table.items.iter().map(|i| {
            Row::new([
                // Cell::new("".to_owned()).fg(Color::Yellow),
                Cell::new("".to_owned()).fg(Color::Red),
                Cell::new(i.from.to_owned()),
                Cell::new(
                    i.subject
                        .chars()
                        .filter(|x| x.width().is_some_and(|x| x != 0)) // Filter 0 width chars
                        .collect::<String>(),
                )
                .italic()
                .fg(Color::Gray),
                Cell::new(i.date.to_owned()),
            ])
            .white()
        });
        let widths = [
            Constraint::Length(1),
            Constraint::Max(15),
            Constraint::Min(25),
            Constraint::Max(12),
        ];
        let focus_style = match ctx.mode == Mode::Focus(Focusable::Emails) {
            true => Style::new().light_cyan(),
            false => Style::new().white(),
        };
        let focus_right = match ctx.mode {
            Mode::Focus(Focusable::Emails) | Mode::Focus(Focusable::Preview) => {
                Style::new().light_cyan()
            }
            _ => Style::new().white(),
        };
        let focus_left = match ctx.mode {
            Mode::Focus(Focusable::Emails) | Mode::Focus(Focusable::Sidebar) => {
                Style::new().light_cyan()
            }
            _ => Style::new().white(),
        };

        let border = Block::default().borders(Borders::ALL).style(focus_style);
        let border_right = Block::default().borders(Borders::RIGHT).style(focus_right);
        let border_left = Block::default().borders(Borders::LEFT).style(focus_left);
        let hdown = Paragraph::new(line::HORIZONTAL_DOWN);
        let hup = Paragraph::new(line::HORIZONTAL_UP);
        let left_area = Rect::new(area.x, area.y, 1, area.height);
        let right_area = Rect::new(area.x + area.width - 1, area.y, 1, area.height);
        let hdown_left = Rect::new(area.x, area.y, 1, 1);
        let hdown_right = Rect::new(area.x + area.width - 1, area.y, 1, 1);
        let hup_left = Rect::new(area.x, area.y + area.height - 1, 1, 1);
        let hup_right = Rect::new(area.x + area.width - 1, area.y + area.height - 1, 1, 1);
        f.render_widget(Clear, area);
        f.render_stateful_widget(
            Table::new(rows, widths)
                .block(border.title(title))
                .highlight_style(Style::new().bg(Color::DarkGray))
                .fg(Color::White)
                .header(
                    Row::new(["", "From", "Subject", "Date"])
                        .underlined()
                        .fg(Color::LightCyan),
                ),
            area,
            &mut self.table.state,
        );
        f.render_widget(border_right, right_area);
        f.render_widget(border_left, left_area);
        f.render_widget(hdown.clone().style(focus_right), hdown_right);
        f.render_widget(hdown.style(focus_left), hdown_left);
        f.render_widget(hup.clone().style(focus_right), hup_right);
        f.render_widget(hup.style(focus_left), hup_left);

        if ctx.mode == Mode::Focus(Focusable::Emails) {
            let sb = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .track_symbol(Some("│"))
                .begin_symbol(Some(""))
                .end_symbol(None);
            let sb_area = Rect::new(area.x, area.y + 1, area.width, area.height - 2);
            f.render_stateful_widget(sb, sb_area, &mut self.table.scrollbar_state);
        }
        self.last_area = area;
    }

    fn on(&mut self, e: Event) -> Option<Mode> {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Char('k') => {
                    self.table.next(-1);
                }
                KeyCode::Char('j') => {
                    self.table.next(1);
                }
                KeyCode::Char('g') => {
                    self.table.first();
                }
                KeyCode::Char('G') => {
                    self.table.last();
                }
                KeyCode::Enter => {
                    return Some(Mode::Loading(LoadType::FetchPreview));
                }
                KeyCode::Char(' ') => {
                    return Some(Mode::Loading(LoadType::FetchPreview));
                }
                _ => {}
            }
        } else if let Event::Mouse(MouseEvent {
            kind, row, column, ..
        }) = e
        {
            match kind {
                MouseEventKind::ScrollUp => {
                    if self.last_area.contains(Position::new(column, row)) {
                        match self.table.state.offset() {
                            2.. => *self.table.state.offset_mut() -= 2,
                            1 => *self.table.state.offset_mut() = 0,
                            _ => {}
                        }
                        self.table.next(-2);
                    }
                }
                MouseEventKind::ScrollDown => {
                    if self.last_area.contains(Position::new(column, row)) {
                        *self.table.state.offset_mut() += 2;
                        self.table.next(2);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn help(self) -> Option<(&'static str, &'static str)> {
        todo!()
    }
}
