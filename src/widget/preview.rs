use std::cmp;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use html2text::render::text_renderer::RichAnnotation;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::{Context, LoadType, Mode};

use super::{focus_border, Focusable, Widget};

// struct StyledLine {
//     content: String,
//     style: Style,
//     img_src: Option<String>,
//     href: Option<String>
// }

#[derive(Default)]
pub struct Preview {
    content: String,
    state: u16,
    content_length: u16,
    scroll: ScrollbarState,
}

fn raw_to_lines<'a>(raw_html: String, width: usize) -> Vec<Line<'a>> {
    let bytes = raw_html.as_bytes();
    let body = html2text::from_read_rich(bytes, width - 2);
    let mut lines: Vec<Line> = vec![];
    for line in body {
        let mut spans: Vec<Span> = vec![];
        for ts in line.tagged_strings() {
            let s = ts.s.replace(['\u{200C}', '\u{200B}'], " ");
            let mut style = Style::new();
            let mut link: Option<String> = None;
            let mut img: Option<String> = None;
            for tag in ts.tag.clone() {
                match tag {
                    RichAnnotation::Emphasis => style = style.italic(),
                    // RichAnnotation::Preformat(b) => style = style,
                    RichAnnotation::Code => style = style.bg(Color::DarkGray).fg(Color::White),
                    RichAnnotation::Strong => style = style.bold(),
                    RichAnnotation::Default => (),
                    RichAnnotation::Link(href) => {
                        // TODO: Somehow store href
                        // style = style.underlined().fg(Color::LightBlue)
                        link = Some(href);
                    }
                    RichAnnotation::Strikeout => style = style.crossed_out(),
                    RichAnnotation::Image(src) => {
                        // TODO: Somehow store src, maybe display
                        // with rataui-image crate
                        // style = style.underlined().fg(Color::LightGreen)
                        img = Some(src);
                    }
                    RichAnnotation::Colour(c) => style = style.fg(Color::Rgb(c.r, c.g, c.b)),
                    RichAnnotation::BgColour(c) => style = style.bg(Color::Rgb(c.r, c.g, c.b)),
                    _ => (),
                };
            }
            if link.is_some() {
                style = style.fg(Color::LightBlue).underlined();
            }
            if img.is_some() {
                style = style.fg(Color::LightGreen);
            }
            spans.push(Span::styled(s, style));
        }
        lines.push(Line::from(spans));
    }
    lines
}

impl Preview {
    pub fn set_content(&mut self, raw_html: String) {
        self.content = raw_html;
    }
}

impl Widget for Preview {
    fn draw(&mut self, f: &mut Frame, area: Rect, ctx: &mut Context) {
        let lines = raw_to_lines(self.content.clone(), area.width as usize);
        self.content_length = lines.len() as u16;
        let border = focus_border(ctx, Focusable::Preview)
            .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT);
        let title = match ctx.mode == Mode::Loading(LoadType::FetchPreview) {
            true => "Loading...",
            false => "Preview",
        };
        f.render_widget(
            Paragraph::new(lines)
                .scroll((self.state, 0))
                .block(border.title(title)),
            area,
        );
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area,
            &mut self
                .scroll
                .content_length(self.content_length as usize)
                .position(self.state as usize),
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
                KeyCode::Char('k') => self.state = cmp::max(1, self.state) - 1,
                KeyCode::Char('j') => {
                    self.state = cmp::min(cmp::max(1, self.content_length) - 1, self.state + 1)
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
