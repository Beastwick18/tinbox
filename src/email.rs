use std::{error::Error, net::TcpStream};

use imap::Session;
use native_tls::TlsStream;

use crate::config::Config;

pub type TlsSession = Session<TlsStream<TcpStream>>;

pub fn new_session(conf: Config) -> Result<TlsSession, Box<dyn Error>> {
    let domain = conf.imap_server;
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    let client = imap::connect((domain.clone(), conf.imap_port), domain, &tls).unwrap();

    let x = client
        .login(conf.username, conf.password)
        .map_err(|e| e.0)?;
    return Ok(x);
}

pub fn top_messages<'a>(
    session: &mut TlsSession,
    n: u32,
) -> imap::error::Result<Option<Vec<String>>> {
    let mb = session.select("INBOX")?;
    let exists = mb.exists;

    let query = format!("{}:*", exists - n);
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
                msg.subject().map(|s| s.to_owned()).unwrap_or_default()
            })
            .collect(),
    ))
}

pub fn get_html<'a>(session: &mut TlsSession, n: u32) -> imap::error::Result<Option<String>> {
    let mb = session.select("INBOX")?;
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

fn list_inboxes(s: &mut TlsSession) -> Result<Vec<String>, Box<dyn Error>> {
    let l = s.list(None, Some("*"))?;
    let inboxes = l.iter().map(|i| i.name().to_owned()).collect();
    Ok(inboxes)
}