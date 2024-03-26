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
    widget::{emails::Email, preview::Preview, Widget},
};

pub enum LoadType {
    FetchSubjects,
    FetchPreview,
    Login,
}

pub enum Mode {
    ShowEmail,
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
    pub config: Config,
    pub mode: Mode,
    session: Option<TlsSession>,
    errors: VecDeque<String>,
}

#[derive(Default)]
pub struct Widgets {
    preview: Preview,
    email: Email,
    // sidebar: Sidebar,
}

impl App {
    pub fn draw(&mut self, f: &mut Frame) {
        let layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Length(24),
                Constraint::Min(24),
                Constraint::Min(50),
            ],
        )
        .split(f.size());

        self.widgets.email.draw(f, layout[1]);
        self.widgets.preview.draw(f, layout[2]);
    }

    pub async fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        let conf = match Config::load() {
            Ok(c) => c,
            Err(e) => return Err(e.into()),
        };
        conf.apply(self);
        self.mode = Mode::Loading(LoadType::Login);
        while !self.should_quit {
            if !self.errors.is_empty() {
                self.mode = Mode::Error(self.errors.pop_front().unwrap_or_default());
            }

            // get_help(app, w);
            terminal.draw(|f| self.draw(f))?;
            if let Mode::Loading(load) = &self.mode {
                match load {
                    LoadType::FetchSubjects => {
                        self.mode = Mode::ShowEmail;
                        let mut session = match &mut self.session {
                            Some(s) => s,
                            None => {
                                self.show_error("Not logged in");
                                self.mode = Mode::ShowEmail;
                                continue;
                            }
                        };
                        let subs = match email::top_messages(&mut session, 10) {
                            Ok(body) => body,
                            Err(e) => return Err(e.into()),
                        };
                        let subs = match subs {
                            Some(body) => body,
                            None => vec!["No body :)".to_owned()],
                        };
                        self.mode = Mode::Loading(LoadType::FetchPreview);
                        self.widgets.email.set_subjects(subs);
                    }
                    LoadType::FetchPreview => {
                        self.mode = Mode::ShowEmail;
                        let mut session = match &mut self.session {
                            Some(s) => s,
                            None => {
                                self.show_error("Not logged in");
                                continue;
                            }
                        };
                        let text = match email::get_html(&mut session, self.widgets.email.selected)
                        {
                            Ok(body) => body,
                            Err(e) => return Err(e.into()),
                        };
                        let text = match text {
                            Some(body) => body,
                            None => "No body :)".to_owned(),
                        };
                        self.widgets.preview.set_text(text);
                    }
                    LoadType::Login => {
                        self.mode = Mode::ShowEmail;
                        if let Some(s) = &mut self.session {
                            if let Err(e) = s.logout() {
                                self.show_error(e.to_string());
                                self.session = None;
                                continue;
                            }
                        }
                        let s = match new_session(self.config.clone()) {
                            Ok(s) => s,
                            Err(e) => {
                                self.show_error(e.to_string());
                                continue;
                            }
                        };
                        self.session = Some(s);
                        self.mode = Mode::Loading(LoadType::FetchSubjects);
                    }
                }
                continue;
            }

            let evt = event::read()?;
            self.on(evt.clone());
            self.on_widgets(evt);
        }

        if let Some(session) = &mut self.session {
            session.logout()?;
        }

        Ok(())
    }

    fn on(&mut self, e: Event) {
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
                _ => {}
            }
        }
    }

    fn on_widgets(&mut self, e: Event) {
        let new_mode = match self.mode {
            Mode::ShowEmail => self.widgets.email.on(e),
            _ => None,
        };
        if let Some(m) = new_mode {
            self.mode = m;
        }
    }

    pub fn show_error<S: ToString>(&mut self, err: S) {
        self.errors.push_front(err.to_string());
    }
}
