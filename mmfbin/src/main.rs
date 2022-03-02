use std::env;
use dotenv::dotenv;
use log::debug;
use mmflib::{Imap, ImapConf};
use mail_parser::Message;

fn main() {
  dotenv().expect("Create a .env with your config from .env.example");
  let host = env::var("IMAP_HOST").expect("IMAP_HOST");
  let port = env::var("IMAP_PORT").expect("IMAP_PORT").parse::<u16>().expect("Invalid IMAP_PORT");
  let username = env::var("IMAP_USERNAME").expect("IMAP_USERNAME");
  let password = env::var("IMAP_PASSWORD").expect("IMAP_PASSWORD");
  let search = env::var("IMAP_SEARCH").expect("IMAP_SEARCH");

  let mut imap = Imap::new(ImapConf { username, password, host, port });
  let fetches = imap.fetches(search.as_str()).expect("500");
  let messages = fetches.iter().filter_map(|f| imap.parse_fetch(f)).collect::<Vec<Message>>();

  for msg in messages {
    println!("{:?}", msg.get_subject());
    println!("{:?}", msg.get_date().unwrap().to_string());
    println!("{:?}", msg.get_body_preview(150));
    println!("---");
  }

  debug!("DONE");
  imap.logout().expect("500");
}
