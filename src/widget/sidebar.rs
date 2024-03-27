use std::cmp;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::{Context, LoadType, Mode};

use super::{focus_border, Focusable, Widget};

pub struct Sidebar {
    pub selected: u32, // TODO: Use stateful table
    categories: Vec<String>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Sidebar {
            selected: 1,
            categories: vec![],
        }
    }
}

impl Sidebar {
    pub fn set_inboxes(&mut self, cats: Vec<String>) {
        self.categories = cats;
    }
}

impl Widget for Sidebar {
    fn draw(&mut self, f: &mut Frame, area: Rect, ctx: &mut Context) {
        let lines = self
            .categories
            .iter()
            .enumerate()
            .map(|(i, l)| match i == self.selected as usize {
                true => Line::from(l.to_owned()).style(Style::new().bg(Color::DarkGray)),
                false => Line::from(l.to_owned()),
            })
            .collect::<Vec<Line>>();
        let border = focus_border(ctx, Focusable::Sidebar)
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .border_type(BorderType::Rounded);
        let title = match ctx.mode == Mode::Loading(LoadType::FetchInboxes) {
            true => "Loading...",
            false => "Inboxes",
        };
        f.render_widget(Paragraph::new(lines).block(border.title(title)), area);
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
                    self.selected = cmp::max(1, self.selected) - 1;
                }
                KeyCode::Char('j') => {
                    if !self.categories.is_empty() {
                        self.selected =
                            cmp::min(self.categories.len() as u32 - 1, self.selected + 1);
                    }
                }
                KeyCode::Enter => {
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
