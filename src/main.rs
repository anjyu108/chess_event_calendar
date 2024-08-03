use std::env;

use yaml_rust::{Yaml, YamlLoader, YamlEmitter};
use linked_hash_map::LinkedHashMap;
use mysql::*;
use mysql::prelude::*;

const CHESS_CLUB_LIST: [&str; 3] = ["club8x8", "kitasenjyu", "ncs"];

fn main() {
    println!("CHESS_CLUB_LIST: {:?}", CHESS_CLUB_LIST);

    let club8x8_scraper = ChessEventScraperFactory::create("club8x8");

    let events = club8x8_scraper.scrape_event();
    for e in &events {
        println!("================ event ==============");
        println!("date: {:?}", e.date);
        println!("open_time: {:?}", e.open_time);
        println!("revenue: {:?}", e.revenue);
        println!("fee: {:?}", e.fee);
    }

    save_events_to_db(events).unwrap();
}

struct ChessEventScraperFactory;

impl ChessEventScraperFactory {
    fn create(keyword: &str) -> impl ChessEventScraper {
        // TODO add other than 8x8 club scraper
        EventScraperClub8x8 {}
    }
}

trait ChessEventScraper {
    fn url() -> String;
    fn scrape_event(&self) -> Vec<EventInfo>;
}

struct EventScraperClub8x8;
impl ChessEventScraper for EventScraperClub8x8 {
    fn url() -> String{
        "https://8by8.hatenablog.com/".to_string()
    }
    fn scrape_event(&self) -> Vec<EventInfo> {
        let body = reqwest::blocking::get(EventScraperClub8x8::url()).unwrap().text().unwrap();
        let document = scraper::Html::parse_document(&body);

        let div_selector = scraper::Selector::parse("article").unwrap();
        let div_elements = document.select(&div_selector);
        let p_selector = scraper::Selector::parse("p").unwrap();

        let mut events = Vec::new();
        for div in div_elements {
            let mut date = String::from("");
            let mut open_time = String::from("");
            let mut revenue = String::from("");
            let mut fee = String::from("");
            // ================ article ==============
            for e in div.select(&p_selector) {
                // println!("yaxu 4, {:?}", div);
                let text = e.text().collect::<Vec<_>>().join("");
                if text.contains("場所:") {
                    revenue = String::from(text.trim().trim_start_matches("場所:").trim());
                }
                if text.contains("日時:") {
                    date = String::from(text.trim().trim_start_matches("日時:").trim());
                    date = trim_left(
                        &date,
                        Vec::from([String::from("(定員"), String::from("（定員")]),
                    );
                }
                if text.contains("参加費:") {
                    fee = String::from(text.trim().trim_start_matches("参加費:").trim());
                }
                let re = regex::Regex::new(r"(\d{2})時\d{2}分〜\d{2}時\d{2}分").unwrap();
                if re.is_match(&text) {
                    open_time = String::from(text.trim());
                }
            }

            if date == "" ||
               open_time == "" ||
               revenue == "" ||
               fee == "" {
                continue;
            }

            let e = EventInfo {
                date,
                open_time,
                revenue,
                fee,
            };
            events.push(e);
        }

        events
    }
}


pub struct EventInfo {
    date: String,
    open_time: String,
    revenue: String,
    fee: String,
}

pub trait ChessClub {
    fn to_yaml(&self) -> String;
    fn name(&self) -> &String;
    fn url(&self) -> &String;
    fn scrape_event(&self) -> Vec<EventInfo>;
}

struct ChessClub8x8 {
    _name: String,
    _url: String,
}

struct ChessClubKitaSenjyu {
    _name: String,
    _url: String,
}

impl ChessClub for ChessClubKitaSenjyu {
    fn to_yaml(&self) -> String {
        "".to_string()
    }
    fn name(&self) -> &String {
        &self._name
    }
    fn url(&self) -> &String {
        &self._url
    }
    fn scrape_event(&self) -> Vec<EventInfo> {
        let body = reqwest::blocking::get(self.url()).unwrap().text().unwrap();
        let document = scraper::Html::parse_document(&body);

        let div_selector = scraper::Selector::parse("div.entry-content").unwrap();
        let div_elements = document.select(&div_selector);
        let p_selector = scraper::Selector::parse("p").unwrap();

        let mut events = Vec::new();
        for div in div_elements {
            // log::info!("================");
            let mut date = String::from("");
            let mut open_time = String::from("");
            let mut revenue = String::from("");
            let mut fee = String::from("");
            for e in div.select(&p_selector) {
                let text = e.text().collect::<Vec<_>>().join("");
                // log::info!("text: {:?}", text);

                if text.contains("場所:") {
                    revenue = String::from(text.trim().trim_start_matches("場所:").trim());
                }
                if text.contains("日時:") {
                    date = String::from(text.trim().trim_start_matches("日時:").trim());
                    date = trim_left(
                        &date,
                        Vec::from([String::from("(定員"), String::from("（定員")]),
                    );
                }
                if text.contains("参加費:") {
                    fee = String::from(text.trim().trim_start_matches("参加費:").trim());
                }
                let re = regex::Regex::new(r"(\d{2})時\d{2}分〜\d{2}時\d{2}分").unwrap();
                if re.is_match(&text) {
                    open_time = String::from(text.trim());
                    // log::info!("open_time: {:?}", open_time);
                } else {
                    // log::info!("NOT open_time: {:?}", text);
                }
            }

            let e = EventInfo {
                date,
                open_time,
                revenue,
                fee,
            };
            events.push(e);
        }

        events
    }
}

fn trim_left(text: &str, patterns: Vec<String>) -> String {
    let mut ret = text;
    for p in patterns {
        ret = match ret.find(&p) {
            Some(val) => ret[..val].trim(),
            None => ret,
        };
    }

    String::from(ret)
}

fn save_events_to_db(events: Vec<EventInfo>) -> Result<()> {
    let db_user = env::var("DB_USER").expect("DB_USER must be set");
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");
    let db_socket = env::var("DB_SOCKET").ok(); // ソケットファイルはオプション

    let opts = if let Some(socket) = db_socket {
        OptsBuilder::default()
            .user(Some(db_user))
            .pass(Some(db_password))
            .db_name(Some(db_name))
            .socket(Some(socket))
    } else {
        let db_host = env::var("DB_HOST").expect("DB_HOST must be set");
        let db_port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string()); // デフォルトポートを設定
        OptsBuilder::default()
            .ip_or_hostname(Some(db_host))
            .user(Some(db_user))
            .pass(Some(db_password))
            .db_name(Some(db_name))
            .tcp_port(db_port.parse().expect("DB_PORT must be a valid number"))
    };

    let pool = Pool::new(opts)?;
    let mut conn = pool.get_conn()?;

    for event in events {
        conn.exec_drop(
            r"INSERT INTO chess_event (date, open_time, revenue, fee)
              VALUES (:date, :open_time, :revenue, :fee)",
            params! {
                "date" => event.date,
                "open_time" => event.open_time,
                "revenue" => event.revenue,
                "fee" => event.fee,
            },
        )?;
    }
    Ok(())
}
