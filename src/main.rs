use std::env;

use mysql::{params, OptsBuilder, Pool};
use mysql::prelude::Queryable;

const CHESS_CLUB_LIST: [&str; 2] = ["8x8_chess_club", "kitasenjyu"];

fn main() {
    println!("CHESS_CLUB_LIST: {:?}", CHESS_CLUB_LIST);

    for chess_club_keyword in CHESS_CLUB_LIST {
        let scraper = ChessEventScraperFactory::create(chess_club_keyword);
        match scraper {
            Ok(s) => output_event_list(chess_club_keyword, s.scrape_event()),
            Err(e) => println!("scraper setup error: {:?}", e),
        }
    }
}

struct ChessEventScraperFactory;

impl ChessEventScraperFactory {
    fn create(keyword: &str) -> Result<Box<dyn ChessEventScraper>, &'static str> {
        match keyword {
            "8x8_chess_club" => 
                Ok(Box::new(EventScraperClub8x8 {}) as Box<dyn ChessEventScraper>),
            "kitasenjyu" => 
                Ok(Box::new(EventScraperKitasenjyu{}) as Box<dyn ChessEventScraper>),
            _ => Err("Not supported keyword")
        }
    }
}

trait ChessEventScraper {
    fn scrape_event(&self) -> Vec<EventInfo>;
}

struct EventScraperClub8x8;
impl ChessEventScraper for EventScraperClub8x8 {
    fn scrape_event(&self) -> Vec<EventInfo> {
        let url = "https://8by8.hatenablog.com/".to_string();
        let body = 
            reqwest::blocking::get(url)
            .unwrap().text().unwrap();
        let document = scraper::Html::parse_document(&body);

        let scrape_target_selector = 
            scraper::Selector::parse("article").unwrap();
        let scrape_target_list = document.select(&scrape_target_selector);
        let paragraph_selector = scraper::Selector::parse("p").unwrap();

        let mut events = Vec::new();
        for article in scrape_target_list {
            let mut date = String::from("");
            let mut open_time = String::from("");
            let mut revenue = String::from("");
            let mut fee = String::from("");
            // ================ article ==============
            for paragraph in article.select(&paragraph_selector) {
                // TODO: Fix these duplicated code
                let text = paragraph.text().collect::<Vec<_>>().join("");
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

            // not or not well structured event
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

struct EventScraperKitasenjyu;
impl ChessEventScraper for EventScraperKitasenjyu {
    fn scrape_event(&self) -> Vec<EventInfo> {
        let url = "http://chess.m1.valueserver.jp/".to_string();
        let charset = "Shift_JIS";  // need to manually set charset
        let body = 
            reqwest::blocking::get(url)
            .unwrap().text_with_charset(charset).unwrap();
        let document = scraper::Html::parse_document(&body);

        let scrape_target_selector = 
            scraper::Selector::parse("div.item").unwrap();
        let scrape_target_list = document.select(&scrape_target_selector);

        let mut events = Vec::new();
        for article in scrape_target_list {
            let text = article.text().collect::<Vec<_>>().join("");

            if !text.contains("公式戦例会予定") {
                continue;
            }

            let mut date = String::from("");
            let mut open_time = String::from("");
            let mut revenue = String::from("");
            let mut fee = String::from("");

            // TODO fix this not clean code
            for line in text.lines() {
                if !line.contains("日本チェス連盟公式戦２Ｒ") {
                    continue
                }

                let infos: Vec<&str> = line.split("\u{3000}").collect();
                date = infos[1].to_string();
                open_time = infos[2].to_string();
                revenue = infos[3].to_string();
                fee = "unknown".to_string()
            }

            // not or not well structured event
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


fn output_event_list(title: &str, events: Vec<EventInfo>) {
    print_event_list(title, &events);
    let _ = save_events_to_db(events);
}

fn print_event_list(title: &str, events: &Vec<EventInfo>) {
    println!("============ {:?} ============", title);
    for e in events {
        println!("event");
        println!("  - date: {:?}", e.date);
        println!("  - open_time: {:?}", e.open_time);
        println!("  - revenue: {:?}", e.revenue);
        println!("  - fee: {:?}", e.fee);
    }
    println!("");
}

fn save_events_to_db(events: Vec<EventInfo>) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Change events type to &XX
    let db_user = env::var("DB_USER").expect("DB_USER must be set");
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");
    let db_socket = env::var("DB_SOCKET").ok();

    let opts = if let Some(socket) = db_socket {
        OptsBuilder::default()
            .user(Some(db_user))
            .pass(Some(db_password))
            .db_name(Some(db_name))
            .socket(Some(socket))
    } else {
        let db_host = env::var("DB_HOST").expect("DB_HOST must be set");
        let db_port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
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
