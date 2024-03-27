use std::{cmp, slice::Iter};

use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize as _},
    widgets::{Block, Borders, Clear, ScrollbarState, TableState, Widget as _},
    Frame,
};

use crate::app::{Context, Mode};

use self::{emails::Emails, preview::Preview, search::Search, sidebar::Sidebar};

pub mod emails;
pub mod preview;
pub mod search;
pub mod sidebar;

#[derive(Clone, PartialEq, Eq)]
pub enum Focusable {
    Search,
    Sidebar,
    Subjects,
    Preview,
}

#[derive(Default)]
pub struct Widgets {
    pub search: Search,
    pub sidebar: Sidebar,
    pub email: Emails,
    pub preview: Preview,
}

impl Widgets {
    pub fn on(&mut self, e: Event, ctx: &mut Context) {
        let new_mode = match ctx.mode.clone() {
            Mode::Focus(f) => match f {
                Focusable::Search => self.search.on(e),
                Focusable::Subjects => self.email.on(e),
                Focusable::Sidebar => self.sidebar.on(e),
                Focusable::Preview => self.preview.on(e),
            },
            _ => None,
        };
        if let Some(m) = new_mode {
            ctx.mode = m;
        }
    }
}

pub trait EnumIter<T> {
    fn iter() -> Iter<'static, T>;
}

// pub fn centered_rect(mut x_len: u16, mut y_len: u16, r: Rect) -> Rect {
//     x_len = cmp::min(x_len, r.width);
//     y_len = cmp::min(y_len, r.height);
//     let popup_layout = Layout::new(
//         Direction::Vertical,
//         [
//             Constraint::Length((r.height - y_len) / 2),
//             Constraint::Length(y_len),
//             Constraint::Length((r.height - y_len) / 2),
//         ],
//     )
//     .split(r);
//
//     Layout::new(
//         Direction::Horizontal,
//         [
//             Constraint::Length((r.width - x_len) / 2),
//             Constraint::Length(x_len),
//             Constraint::Length((r.width - x_len) / 2),
//         ],
//     )
//     .split(popup_layout[1])[1]
// }

// pub fn clear(area: Rect, buf: &mut Buffer, fill: Color) {
//     Clear.render(area, buf);
//     Block::new().bg(fill).render(area, buf);
// }

fn focus_border(ctx: &Context, f: Focusable) -> Block {
    Block::new()
        .borders(Borders::ALL)
        .border_style(match ctx.mode == Mode::Focus(f) {
            true => Style::new().light_cyan(),
            false => Style::new(),
        })
}

pub trait Widget {
    fn draw(&mut self, f: &mut Frame, area: Rect, ctx: &mut Context);
    fn on(&mut self, e: Event) -> Option<Mode>;
    fn help(self) -> Option<(&'static str, &'static str)>;
}

pub struct StatefulTable<T> {
    pub state: TableState,
    pub scrollbar_state: ScrollbarState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::new().with_selected(Some(0)),
            scrollbar_state: ScrollbarState::new(items.len()),
            items,
        }
    }

    pub fn next_wrap(&mut self, amt: isize) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => (i as isize + amt).rem_euclid(self.items.len() as isize),
            None => 0,
        };
        self.state.select(Some(i as usize));
        self.scrollbar_state = self.scrollbar_state.position(i as usize);
    }

    pub fn next(&mut self, amt: isize) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => i as isize + amt,
            None => 0,
        };
        let idx = i.max(0).min(self.items.len() as isize - 1) as usize;
        self.state.select(Some(idx));
        self.scrollbar_state = self.scrollbar_state.position(idx);
    }

    pub fn select(&mut self, idx: usize) {
        self.state.select(Some(idx));
        self.scrollbar_state = self.scrollbar_state.position(idx);
    }
}
