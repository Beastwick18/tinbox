use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{BorderType, Borders, Row, Table},
    Frame,
};

use crate::app::{Context, LoadType, Mode};

use super::{focus_border, Focusable, StatefulTable, Widget};

pub struct Sidebar {
    pub table: StatefulTable<String>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Sidebar {
            table: StatefulTable::new(),
        }
    }
}

impl Sidebar {
    pub fn set_inboxes(&mut self, cats: Vec<String>) {
        self.table.items = cats;
        self.table.select(1);
    }
}

impl Widget for Sidebar {
    fn draw(&mut self, f: &mut Frame, area: Rect, ctx: &mut Context) {
        // let lines = self
        //     .table
        //     .items
        //     .iter()
        //     .enumerate()
        //     .map(|(i, l)| match i == self.selected as usize {
        //         true => Line::from(l.to_owned()).style(Style::new().bg(Color::DarkGray)),
        //         false => Line::from(l.to_owned()),
        //     })
        //     .collect::<Vec<Line>>();
        let lines: Vec<Row> = self
            .table
            .items
            .iter()
            .map(|i| Row::new([i.to_owned()]))
            .collect();
        let border = focus_border(ctx, Focusable::Sidebar)
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .border_type(BorderType::Rounded);
        let title = match ctx.mode == Mode::Loading(LoadType::FetchInboxes) {
            true => "Loading...",
            false => "Inboxes",
        };
        f.render_stateful_widget(
            Table::new(lines, [Constraint::Percentage(100)])
                .block(border.title(title))
                .highlight_style(Style::new().bg(Color::DarkGray)),
            area,
            &mut self.table.state,
        );
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
                    return Some(Mode::Loading(LoadType::FetchEmails));
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
