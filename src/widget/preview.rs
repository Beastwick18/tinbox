use core::time;
use std::{borrow::Borrow, cmp, ops::Deref};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use html2text::{
    render::text_renderer::{RichAnnotation, RichDecorator, TaggedLine, TextDecorator},
    RenderTree,
};
use ratatui::{
    layout::{Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use regex::Regex;

use crate::app::{Context, LoadType, Mode};

use super::{focus_border, Focusable, Widget};

#[derive(Clone)]
struct StyledSpan {
    content: String,
    style: Option<Style>,
    img_src: Option<String>,
    href: Option<String>,
}

impl StyledSpan {
    fn new(content: String) -> StyledSpan {
        StyledSpan {
            content,
            style: None,
            img_src: None,
            href: None,
        }
    }

    fn style(&mut self, style: Style) -> &mut Self {
        self.style = Some(style);
        self
    }

    fn href(&mut self, href: String) -> &mut Self {
        self.href = Some(href);
        self
    }

    fn src(&mut self, src: String) -> &mut Self {
        self.img_src = Some(src);
        self
    }
}

// struct StyledLine {}
type StyledLine = Vec<StyledSpan>;

fn to_lines<'a>(lines: Vec<StyledLine>) -> Vec<Line<'a>> {
    lines
        .iter()
        .map(|l| {
            let spans: Vec<Span> = l
                .iter()
                .map(|s| Span::styled(s.content.to_owned(), s.style.unwrap_or_default()))
                .collect();
            Line::from(spans)
        })
        .collect()
}

type Lines = Vec<TaggedLine<Vec<<RichDecorator as TextDecorator>::Annotation>>>;

#[derive(Default)]
pub struct Preview {
    content: Option<RenderTree>,
    state: u16,
    content_length: u16,
    scroll: ScrollbarState,
    last_area: Rect,
    redraw: bool,
    lines: Vec<StyledLine>,
}

fn raw_to_lines(body: &Lines) -> Vec<StyledLine> {
    // let body = content.render_rich(width).unwrap().into_lines().unwrap();
    let mut lines: Vec<StyledLine> = vec![];
    for line in body {
        let mut spans: StyledLine = vec![];
        for ts in line.tagged_strings() {
            let re = Regex::new(r"\s").unwrap();
            let s = re.replace_all(&ts.s, " ").to_string();
            // let s = ts.s.replace(['\u{200C}', '\u{200B}'], " ");
            let mut style = Style::new();
            let mut span = StyledSpan::new(s);
            for tag in ts.tag.clone() {
                match tag {
                    RichAnnotation::Emphasis => style = style.italic(),
                    // RichAnnotation::Preformat(b) => style = style,
                    RichAnnotation::Code => style = style.bg(Color::DarkGray).fg(Color::White),
                    RichAnnotation::Strong => style = style.bold(),
                    RichAnnotation::Default => (),
                    RichAnnotation::Link(href) => {
                        span.href(href);
                    }
                    RichAnnotation::Strikeout => style = style.crossed_out(),
                    RichAnnotation::Image(src) => {
                        span.src(src);
                    }
                    RichAnnotation::Colour(c) => style = style.fg(Color::Rgb(c.r, c.g, c.b)),
                    RichAnnotation::BgColour(c) => style = style.bg(Color::Rgb(c.r, c.g, c.b)),
                    _ => (),
                };
            }
            if span.href.is_some() {
                style = style.fg(Color::LightBlue).underlined();
            }
            if span.img_src.is_some() {
                style = style.fg(Color::LightGreen);
            }
            spans.push(span.style(style).to_owned());
        }
        lines.push(spans);
    }
    lines
}

impl Preview {
    pub fn set_content(&mut self, raw_html: String) {
        self.content = html2text::parse(raw_html.as_bytes()).ok();
        self.redraw = true;
        self.state = 0;
    }
}

impl Widget for Preview {
    fn draw(&mut self, f: &mut Frame, area: Rect, ctx: &mut Context) {
        // let mut lines = vec![];
        if self.last_area != area || self.redraw {
            if let Some(c) = self.content.to_owned() {
                let rendered_lines =
                    html2text::config::rich().render_to_lines(c, area.width as usize - 2);
                if let Ok(c) = rendered_lines {
                    self.lines = raw_to_lines(&c);
                    self.redraw = false;
                    self.last_area = area;
                }
            }
        }
        self.content_length = self.lines.len() as u16;
        let border = focus_border(ctx, Focusable::Preview)
            .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
            .border_type(BorderType::Rounded);
        let title = match ctx.mode == Mode::Loading(LoadType::FetchPreview) {
            true => "Loading...",
            false => "Preview",
        };
        f.render_widget(
            Paragraph::new(to_lines(self.lines.to_owned()))
                .scroll((self.state, 0))
                .block(border.title(title)),
            area,
        );
        let sb = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("│"))
            .begin_symbol(None)
            .end_symbol(None);
        let sb_area = Rect::new(area.x, area.y + 1, area.width, area.height - 2);
        let visible_height = cmp::max(2, area.height) - 2;
        let length = cmp::max(self.content_length, visible_height) - visible_height;
        self.content_length = length + 1;
        f.render_stateful_widget(
            sb,
            sb_area,
            &mut self
                .scroll
                .content_length(length as usize)
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
        } else if let Event::Mouse(MouseEvent {
            kind, row, column, ..
        }) = e
        {
            match kind {
                MouseEventKind::ScrollUp => {
                    if self.last_area.contains(Position::new(column, row)) {
                        self.state = cmp::max(2, self.state) - 2;
                    }
                }
                MouseEventKind::ScrollDown => {
                    if self.last_area.contains(Position::new(column, row)) {
                        self.state = cmp::min(cmp::max(1, self.content_length) - 1, self.state + 2);
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
