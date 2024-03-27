use std::{collections::VecDeque, error::Error};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

use crate::{
    config::Config,
    email::{self, new_session, TlsSession},
    widget::{Focusable, Widget, Widgets},
};

#[derive(Clone, PartialEq, Eq)]
pub enum LoadType {
    FetchSubjects,
    FetchPreview,
    FetchInboxes,
    Login,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Mode {
    Focus(Focusable),
    Loading(LoadType),
    Error(String),
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Loading(LoadType::FetchSubjects)
    }
}

#[derive(Default)]
pub struct App {
    pub widgets: Widgets,
    should_quit: bool,
}

#[derive(Default)]
pub struct Context {
    pub mode: Mode,
    pub config: Config,
    pub errors: VecDeque<String>,
    session: Option<TlsSession>,
}

impl Context {
    pub fn show_error<S: ToString>(&mut self, err: S) {
        self.errors.push_front(err.to_string());
    }
}

impl App {
    pub fn draw(&mut self, f: &mut Frame, ctx: &mut Context) {
        let layout_vert = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Min(1)],
        )
        .split(f.size());
        let layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Length(24),
                Constraint::Min(24),
                Constraint::Min(50),
            ],
        )
        .split(layout_vert[1]);

        self.widgets.search.draw(f, layout_vert[0], ctx);
        self.widgets.sidebar.draw(f, layout[0], ctx);
        self.widgets.email.draw(f, layout[1], ctx);
        self.widgets.preview.draw(f, layout[2], ctx);
    }

    pub async fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();
        let conf = match Config::load() {
            Ok(c) => c,
            Err(e) => return Err(e.into()),
        };
        conf.apply(&mut ctx);
        ctx.mode = Mode::Loading(LoadType::Login);
        while !self.should_quit {
            if !ctx.errors.is_empty() {
                ctx.mode = Mode::Error(ctx.errors.pop_front().unwrap_or_default());
            }

            // get_help(app, w);
            terminal.draw(|f| self.draw(f, &mut ctx))?;
            if let Mode::Loading(load) = &ctx.mode {
                let load = load.clone();
                ctx.mode = Mode::Focus(Focusable::Subjects);
                match load {
                    LoadType::FetchSubjects => {
                        let session = match &mut ctx.session {
                            Some(s) => s,
                            None => {
                                ctx.show_error("Not logged in");
                                continue;
                            }
                        };
                        ctx.mode = Mode::Focus(Focusable::Subjects);
                        let subs = match email::top_messages(session, 100) {
                            Ok(body) => body,
                            Err(e) => {
                                ctx.show_error(e);
                                continue;
                            }
                        };
                        let subs = match subs {
                            Some(body) => body,
                            None => vec![],
                        };
                        self.widgets.email.set_entries(subs);
                    }
                    LoadType::FetchPreview => {
                        let session = match &mut ctx.session {
                            Some(s) => s,
                            None => {
                                ctx.show_error("Not logged in");
                                continue;
                            }
                        };
                        ctx.mode = Mode::Focus(Focusable::Preview);
                        let text = match email::get_html(
                            session,
                            self.widgets
                                .email
                                .table
                                .state
                                .selected()
                                .unwrap_or_default() as u32,
                        ) {
                            Ok(body) => body,
                            Err(e) => {
                                ctx.show_error(e);
                                continue;
                            }
                        }
                        .unwrap_or_default();
                        self.widgets.preview.set_content(text);
                    }
                    LoadType::FetchInboxes => {
                        let session = match &mut ctx.session {
                            Some(s) => s,
                            None => {
                                ctx.show_error("Not logged in");
                                continue;
                            }
                        };
                        ctx.mode = Mode::Focus(Focusable::Subjects);
                        let inboxes = match email::list_inboxes(session) {
                            Ok(body) => body,
                            Err(e) => {
                                ctx.show_error(e);
                                continue;
                            }
                        };
                        ctx.mode = Mode::Loading(LoadType::FetchSubjects);
                        self.widgets.sidebar.set_inboxes(inboxes);
                    }
                    LoadType::Login => {
                        ctx.mode = Mode::Focus(Focusable::Subjects);
                        if let Some(s) = &mut ctx.session {
                            if let Err(e) = s.logout() {
                                ctx.show_error(e);
                                ctx.session = None;
                                continue;
                            }
                        }
                        let s = match new_session(ctx.config.clone()) {
                            Ok(s) => s,
                            Err(e) => {
                                ctx.show_error(e.to_string());
                                continue;
                            }
                        };
                        ctx.session = Some(s);
                        ctx.mode = Mode::Loading(LoadType::FetchInboxes);
                    }
                }
                continue;
            }

            let evt = event::read()?;
            self.on(evt.clone(), &mut ctx);
            self.widgets.on(evt, &mut ctx);
        }

        if let Some(session) = &mut ctx.session {
            session.logout()?;
        }

        Ok(())
    }

    fn on(&mut self, e: Event, ctx: &mut Context) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('l') | KeyCode::Tab => {
                    if let Mode::Focus(f) = ctx.mode.clone() {
                        ctx.mode = Mode::Focus(match f {
                            Focusable::Search => Focusable::Subjects,
                            Focusable::Sidebar => Focusable::Subjects,
                            Focusable::Subjects => Focusable::Preview,
                            Focusable::Preview => Focusable::Sidebar,
                        });
                    }
                }
                KeyCode::Char('h') | KeyCode::BackTab => {
                    if let Mode::Focus(f) = ctx.mode.clone() {
                        ctx.mode = Mode::Focus(match f {
                            Focusable::Search => Focusable::Subjects,
                            Focusable::Subjects => Focusable::Sidebar,
                            Focusable::Preview => Focusable::Subjects,
                            Focusable::Sidebar => Focusable::Preview,
                        });
                    }
                }
                KeyCode::Char('/') => {
                    if let Mode::Focus(f) = ctx.mode.clone() {
                        if f != Focusable::Search {
                            ctx.mode = Mode::Focus(Focusable::Search);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
