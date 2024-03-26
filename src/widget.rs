use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};

use crate::app::Mode;

pub mod emails;
pub mod preview;
pub mod sidebar;

pub trait Widget {
    fn draw(&mut self, f: &mut Frame, area: Rect);
    fn on(&mut self, e: Event) -> Option<Mode>;
    fn help(self) -> Option<(&'static str, &'static str)>;
}
