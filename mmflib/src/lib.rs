use std::{net::TcpStream};
use imap::{ClientBuilder, Session, types::{Fetches, Fetch}};
use log::{debug, trace};
use mail_parser::Message;
use anyhow::{anyhow, Result, Context};
use native_tls::TlsStream;

type ImapSession = Session<TlsStream<TcpStream>>;

pub struct ImapConf {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
}

pub struct Imap {
    pub conf: ImapConf,
    pub sess: Option<ImapSession>,
}

impl Imap {
    pub fn new(conf: ImapConf) -> Self {
        Self {
            conf,
            sess: None,
        }
    }
    pub fn sess(&mut self) -> Result<&mut ImapSession> {
        if self.sess.is_none() {
            let conf = &self.conf;

            debug!("create client");
            let client = ClientBuilder::new(&conf.host, conf.port)
                .native_tls()
                .context("Unable to create Client")?;

            debug!("create session");
            let mut sess = client
                .login(&conf.username, &conf.password)
                .map_err(|e| e.0)
                .context("Unable to create IMAP session")?;

            sess.select("INBOX")
                .context("Unable to select INBOX")?;
            self.sess = Some(sess);
        }

        return match self.sess {
            Some(ref mut sess) => Ok(sess),
            None => Err(anyhow!("cannot get IMAP session")),
        }
    }

    pub fn uid_search(&mut self, query: &str) -> Result<Vec<String>> {
        let uids: Vec<String> = self
            .sess()?
            .uid_search(query)
            .context("cannot search new messages")?
            .into_iter()
            .map(|x| x.to_string())
            .collect();

        debug!("found {} new messages", uids.len());
        trace!("uids: {:?}", uids);

        Ok(uids)
    }

    pub fn uid_fetch(&mut self, uids: Vec<String>) -> Result<Fetches> {
        self.sess()?.uid_fetch(uids.join(","), "(UID ENVELOPE BODY[])").context("Fetch ERROR")
    }

    pub fn fetches(&mut self, query: &str) -> Result<Fetches> {
        let uids = self.uid_search(query)?;
        let fetches = self.uid_fetch(uids)?;
        return Ok(fetches);
    }

    pub fn parse_fetch<'a>(&mut self, fetch: &'a Fetch) -> Option<Message<'a>> {
        if let Some(b) = fetch.body() {
            if let Some(m) = Message::parse(b) {
                return Some(m);
            }
        }
        return None;
    }

    pub fn logout(&mut self) -> Result<()> {
        if let Some(ref mut sess) = self.sess {
            debug!("logout from IMAP server");
            sess.logout().context("cannot logout from IMAP server")?;
        }
        Ok(())
    }
}
