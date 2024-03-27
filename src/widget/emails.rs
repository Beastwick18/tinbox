use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    symbols::line,
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};
use unicode_width::UnicodeWidthChar as _;

use crate::app::{Context, LoadType, Mode};

use super::{focus_border, Focusable, StatefulTable, Widget};

pub struct EmailEntry {
    pub from: String,
    pub subject: String,
    pub date: String,
}

pub struct Emails {
    pub table: StatefulTable<EmailEntry>,
}

impl Default for Emails {
    fn default() -> Self {
        Emails {
            table: StatefulTable::with_items(vec![]),
        }
    }
}

impl Emails {
    pub fn set_entries(&mut self, entries: Vec<EmailEntry>) {
        self.table.select(0);
        self.table.items = entries;
    }
}

impl Widget for Emails {
    fn draw(&mut self, f: &mut Frame, area: Rect, ctx: &mut Context) {
        let title = match ctx.mode == Mode::Loading(LoadType::FetchSubjects) {
            true => "Loading...".to_owned(),
            false => format!("Emails: {}", ctx.config.username),
        };
        // let rows: Vec<Row> = vec![];
        let rows = self.table.items.iter().map(|i| {
            Row::new([
                i.from.to_owned(),
                i.subject
                    .chars()
                    .filter(|x| x.width().is_some_and(|x| x != 0)) // Filter 0 width chars
                    .collect(),
                i.date.to_owned(),
            ])
            .white()
        });
        let widths = [
            Constraint::Max(15),
            Constraint::Min(25),
            Constraint::Max(10),
        ];
        let focus_style = match ctx.mode == Mode::Focus(Focusable::Subjects) {
            true => Style::new().light_cyan(),
            false => Style::new().white(),
        };

        let border = Block::default().borders(Borders::ALL).style(focus_style);
        let hdown = Paragraph::new(line::HORIZONTAL_DOWN).style(focus_style);
        let hup = Paragraph::new(line::HORIZONTAL_UP).style(focus_style);
        let hdown_left = Rect::new(area.x, area.y, 1, 1);
        let hdown_right = Rect::new(area.x + area.width - 1, area.y, 1, 1);
        let hup_left = Rect::new(area.x, area.y + area.height - 1, 1, 1);
        let hup_right = Rect::new(area.x + area.width - 1, area.y + area.height - 1, 1, 1);
        // Block::new()
        //     .borders(Borders::ALL)
        //     .border_style()
        f.render_stateful_widget(
            Table::new(rows, widths)
                .block(border.title(title))
                .highlight_style(Style::new().bg(Color::DarkGray))
                .fg(Color::White)
                .header(
                    Row::new(["From", "Subject", "Date"])
                        .style(Style::new().fg(Color::LightCyan).underlined()),
                ),
            area,
            &mut self.table.state,
        );
        f.render_widget(hdown.clone(), hdown_right);
        f.render_widget(hdown, hdown_left);
        f.render_widget(hup.clone(), hup_right);
        f.render_widget(hup, hup_left);
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
                KeyCode::Enter => {
                    return Some(Mode::Loading(LoadType::FetchPreview));
                }
                KeyCode::Char(' ') => {
                    return Some(Mode::Loading(LoadType::FetchPreview));
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
