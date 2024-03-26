use crossterm::event::{Event, KeyEvent, KeyEventKind};
use html2text::render::text_renderer::RichAnnotation;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::Mode;

use super::Widget;

#[derive(Default)]
pub struct Preview {
    text: String,
}

impl Preview {
    pub fn set_text(&mut self, body: String) {
        self.text = body;
    }
}

impl Widget for Preview {
    fn draw(&mut self, f: &mut Frame, area: Rect) {
        let bytes = self.text.as_bytes();
        let body = html2text::from_read_rich(&bytes[..], area.width as usize - 4);
        let mut lines: Vec<Line> = vec![];
        for line in body {
            for ts in line.tagged_strings() {
                let s = ts.s.replace("‌", "").replace("​", "").trim_end().to_owned();
                let mut style = Style::new();
                for tag in ts.tag.clone() {
                    match tag {
                        RichAnnotation::Emphasis => style = style.italic(),
                        // RichAnnotation::Preformat(b) => style = style,
                        RichAnnotation::Code => style = style.bg(Color::DarkGray).fg(Color::White),
                        RichAnnotation::Strong => style = style.bold(),
                        RichAnnotation::Default => (),
                        RichAnnotation::Link(_href) => {
                            // TODO: Somehow store href
                            style = style.underlined().fg(Color::LightBlue)
                        }
                        RichAnnotation::Strikeout => style = style.crossed_out(),
                        RichAnnotation::Image(_src) => {
                            // TODO: Somehow store src, maybe display
                            // with rataui-image crate
                            style = style.underlined().fg(Color::LightGreen)
                        }
                        RichAnnotation::Colour(c) => style = style.fg(Color::Rgb(c.r, c.g, c.b)),
                        RichAnnotation::BgColour(c) => style = style.bg(Color::Rgb(c.r, c.g, c.b)),
                        _ => (),
                    };
                }
                lines.push(Line::from(s).style(style))
            }
        }
        f.render_widget(
            Paragraph::new(lines).block(Block::new().borders(Borders::ALL).title("Preview")),
            area,
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
                _ => {}
            }
        }
        None
    }

    fn help(self) -> Option<(&'static str, &'static str)> {
        todo!()
    }
}
