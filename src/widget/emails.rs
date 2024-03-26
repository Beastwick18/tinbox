use std::cmp;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{LoadType, Mode};

use super::Widget;

pub struct Email {
    pub selected: u32, // TODO: Use stateful table
    subjects: Vec<String>,
}

impl Default for Email {
    fn default() -> Self {
        Email {
            selected: 1,
            subjects: vec![],
        }
    }
}

impl Email {
    pub fn set_subjects(&mut self, body: Vec<String>) {
        self.subjects = body;
    }
}

impl Widget for Email {
    fn draw(&mut self, f: &mut Frame, area: Rect) {
        let lines = self
            .subjects
            .iter()
            .enumerate()
            .map(|(i, l)| match i == self.selected as usize {
                true => Line::from(l.to_owned()).style(Style::new().bg(Color::DarkGray)),
                false => Line::from(l.to_owned()),
            })
            .collect::<Vec<Line>>();
        f.render_widget(
            Paragraph::new(lines).block(Block::new().borders(Borders::ALL).title("Emails")),
            area,
        );
    }

    fn on(&mut self, e: crossterm::event::Event) -> Option<Mode> {
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
                    if self.subjects.len() > 0 {
                        self.selected = cmp::min(self.subjects.len() as u32 - 1, self.selected + 1);
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
