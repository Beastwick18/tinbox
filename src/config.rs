use confy::ConfyError;
use serde::{Deserialize, Serialize};

use crate::app::App;

pub static APP_NAME: &str = "tinbox";
pub static CONFIG_FILE: &str = "login";

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub imap_server: String,
    pub imap_port: u16,
}

impl Config {
    pub fn load() -> Result<Config, ConfyError> {
        confy::load::<Config>(APP_NAME, CONFIG_FILE)
    }
    pub fn store(self) -> Result<(), ConfyError> {
        confy::store::<Config>(APP_NAME, CONFIG_FILE, self)
    }
    pub fn apply(&self, app: &mut App) {
        app.config = self.to_owned();
    }
}
