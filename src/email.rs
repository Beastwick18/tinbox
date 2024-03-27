use std::{cmp, error::Error, net::TcpStream};

use imap::Session;
use native_tls::TlsStream;
use unicode_width::UnicodeWidthChar;

use crate::{config::Config, widget::emails::EmailEntry};

pub type TlsSession = Session<TlsStream<TcpStream>>;

pub fn new_session(conf: Config) -> Result<TlsSession, Box<dyn Error>> {
    let domain = conf.imap_server;
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    let client = imap::connect((domain.clone(), conf.imap_port), domain, &tls).unwrap();

    let x = client
        .login(conf.username, conf.password)
        .map_err(|e| e.0)?;
    Ok(x)
}

pub fn top_messages(
    session: &mut TlsSession,
    inbox: String,
    n: u32,
) -> imap::error::Result<Option<Vec<EmailEntry>>> {
    let mb = session.select(inbox)?;
    let from = cmp::max(mb.exists, n) - n;

    let query = format!("{}:*", from);
    let messages = session.fetch(query, "RFC822")?;
    Ok(Some(
        messages
            .iter()
            .rev()
            .map(|message| {
                let body = message.body().expect("message did not have a body!");
                let body = String::from_utf8_lossy(body).to_string();
                let msg = mail_parser::MessageParser::new()
                    .parse(body.as_bytes())
                    .expect("Failed to parse");
                let from = msg
                    .from()
                    .and_then(|f| f.first())
                    .and_then(|f| f.name.clone().or(f.address.clone()))
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                let subject = msg.subject().map(|s| s.to_owned()).unwrap_or_default();
                let subject = subject
                    .chars()
                    .filter(|c| c.width().is_some_and(|c| c != 0)) // Remove 0 width chars
                    .collect();
                let date = "Today".to_owned();
                EmailEntry {
                    from,
                    subject,
                    date,
                }
            })
            .collect(),
    ))
}

pub fn get_html(
    session: &mut TlsSession,
    inbox: String,
    n: u32,
) -> imap::error::Result<Option<String>> {
    let mb = session.select(inbox)?;
    let exists = mb.exists;

    let query = format!("{}", exists - n);
    let messages = session.fetch(query, "RFC822")?;
    Ok(Some(
        messages
            .iter()
            .rev()
            .map(|message| {
                let body = message.body().expect("message did not have a body!");
                let body = std::str::from_utf8(body)
                    .expect("message was not valid utf-8")
                    .to_string();
                let msg = mail_parser::MessageParser::new()
                    .parse(body.as_bytes())
                    .expect("Failed to parse");
                msg.body_html(0).unwrap_or_default().to_string()
            })
            .next() // TODO: Multiple messages??
            .unwrap_or_default(),
    ))
}

pub fn list_inboxes(s: &mut TlsSession) -> Result<Vec<String>, Box<dyn Error>> {
    let l = s.list(None, Some("*"))?;
    let inboxes = l.iter().map(|i| i.name().trim().to_owned()).collect();
    Ok(inboxes)
}
