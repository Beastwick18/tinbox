use crossterm::event::{Event, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    widgets::{BorderType, Paragraph},
    Frame,
};

use crate::app::{Context, Mode};

use super::{focus_border, Focusable, Widget};

pub struct Search {
    pub input: String,
}

impl Default for Search {
    fn default() -> Self {
        Search {
            input: "Testing :)".to_owned(),
        }
    }
}

impl Search {
    // pub fn set_input(&mut self, input: String) {
    //     self.input = input;
    // }
}

impl Widget for Search {
    fn draw(&mut self, f: &mut Frame, area: Rect, ctx: &mut Context) {
        let border = focus_border(ctx, Focusable::Search).border_type(BorderType::Rounded);
        f.render_widget(
            Paragraph::new(ctx.errors.front().map(|x| x.to_owned()).unwrap_or_default())
                .block(border.title("Search")),
            area,
        );
    }

    fn on(&mut self, e: Event) -> Option<Mode> {
        if let Event::Key(KeyEvent {
            // code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            // match code {
            //     _ => {}
            // }
        }
        None
    }

    fn help(self) -> Option<(&'static str, &'static str)> {
        todo!()
    }
}
