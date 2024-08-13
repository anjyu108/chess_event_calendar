use std::env;

use chrono::NaiveDate;

use mysql::{params, OptsBuilder, Pool};
use mysql::prelude::Queryable;

use unicode_normalization::UnicodeNormalization;

use serde::{Deserialize, Serialize};
use serde_yaml;

use std::fs::File;
use std::io::BufReader;

const CHESS_CLUB_LIST: [&str; 3] = ["8x8_chess_club", "kitasenjyu", "JCF"];

// TODO: add logger for debug
// TODO: add unit test

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
            "JCF" =>
                Ok(Box::new(EventScraperJcf{}) as Box<dyn ChessEventScraper>),
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
            let mut date = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
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
                    let date_str = String::from(text.trim().trim_start_matches("日時:").trim());
                    let date_str_trim = trim_left(
                        &date_str,
                        Vec::from([String::from("(定員"), String::from("（定員")]),
                    );
                    date = EventScraperClub8x8::naive_date_from_str(&date_str_trim);
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
            if date == NaiveDate::from_ymd_opt(0, 1, 1).unwrap() ||
               open_time == "" ||
               revenue == "" ||
               fee == "" {
                continue;
            }

            let name = "meeting".to_string();  // Assume all events are meeting
            let organizer = "8x8 chess club".to_string();
            let start_date = date;
            let end_date = date;
            let e = EventInfo {
                name,
                organizer,
                start_date,
                end_date,
                open_time,
                revenue,
                fee,
            };
            events.push(e);
        }

        events
    }
}

impl EventScraperClub8x8 {
    fn naive_date_from_str(input: &str) -> NaiveDate {
        // input example: "2024年1月7日(日)"
        // Return 1995-10-5 when failed to parase

        // normalize input as much as possible (e.g., Zenkaku)
        let input_nfkd = input.nfkd().collect::<String>();

        let only_ymd = trim_left(&input_nfkd, Vec::from([String::from("日")]));

        let year_split: Vec<&str> = only_ymd.split("年").collect();
        let year_str = year_split[0];
        let year_int: i32 = year_str.parse().unwrap_or(1995);

        let month_split: Vec<&str> = year_split[1].split("月").collect();
        let month_str = month_split[0];
        let month_int: u32 = month_str.parse().unwrap_or(10);

        let day_split: Vec<&str> = month_split[1].split("月").collect();
        let day_str = day_split[0];
        let day_int: u32 = day_str.parse().unwrap_or(5);

        let datetime = NaiveDate::from_ymd_opt(year_int, month_int, day_int);

        // FIXME don't use unwrap()
        datetime.unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventInfo {
    name: String,
    organizer: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
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

            let mut date = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
            let mut open_time = String::from("");
            let mut revenue = String::from("");
            let mut fee = String::from("");

            // TODO fix this not clean code
            for line in text.lines() {
                if !line.contains("日本チェス連盟公式戦２Ｒ") {
                    continue
                }

                let infos: Vec<&str> = line.split("\u{3000}").collect();
                let date_str = infos[1].to_string();
                date = EventScraperKitasenjyu::naive_date_from_str(&date_str);
                open_time = infos[2].to_string();
                revenue = infos[3].to_string();
                fee = "unknown".to_string()
            }

            // not or not well structured event
            if date == NaiveDate::from_ymd_opt(0, 1, 1).unwrap() ||
               open_time == "" ||
               revenue == "" ||
               fee == "" {
                continue;
            }

            let name = "meeting".to_string();  // Assume all events are meeting
            let organizer = "kita-senjyu chess club".to_string();
            let start_date = date;
            let end_date = date;
            let e = EventInfo {
                name,
                organizer,
                start_date,
                end_date,
                open_time,
                revenue,
                fee,
            };
            events.push(e);
        }

        events
    }
}

impl EventScraperKitasenjyu {
    fn naive_date_from_str(input: &str) -> NaiveDate {
        // input example: "７／２７（土）"
        // Return 1995-10-5 when failed to parase

        // normalize input as much as possible (e.g., Zenkaku)
        let input_nfkd = input.nfkd().collect::<String>();

        let only_md = trim_left(&input_nfkd, Vec::from([String::from("(")]));

        let year_int = 2024;  // assume it's 2024 year

        let month_split: Vec<&str> = only_md.split("/").collect();
        let month_str = month_split[0];
        let month_int: u32 = month_str.parse().unwrap_or(10);

        let day_str = month_split[1];
        let day_int: u32 = day_str.parse().unwrap_or(5);

        let datetime = NaiveDate::from_ymd_opt(year_int, month_int, day_int);

        // FIXME don't use unwrap()
        datetime.unwrap()
    }
}

struct EventScraperJcf;
impl ChessEventScraper for EventScraperJcf {
    fn scrape_event(&self) -> Vec<EventInfo> {
        // NOTE: JCF's event page is separated foreach year
        //       This is for 2024, and need to change on next year
        let url = "https://japanchess.org/tournament2024-2/".to_string();
        let body = 
            reqwest::blocking::get(url).unwrap().text().unwrap();
        let document = scraper::Html::parse_document(&body);

        let scrape_target_selector = 
            scraper::Selector::parse("div.TournamentBox").unwrap();
        let scrape_target_list = document.select(&scrape_target_selector);
        let paragraph_selector = scraper::Selector::parse("div").unwrap();

        let mut events = Vec::new();
        for article in scrape_target_list {
            // TODO: Fix hardcoded date initial value
            let mut start_date = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
            let mut end_date = NaiveDate::from_ymd_opt(0, 1, 2).unwrap();
            let mut name = String::from("");
            for paragraph in article.select(&paragraph_selector) {
                let text = paragraph.text().collect::<Vec<_>>().join("");
                let attr_class = paragraph.value().attr("class").unwrap_or("");

                if attr_class == "tournamentname" {
                    name = String::from(&text);
                }
                if attr_class == "gamedate" {
                    // text example: "2024/1/7(日)-1/8(月祝)", "2024/1/20(土)"
                    let date_str = String::from(&text);
                    // println!("date_str: {:?}", date_str);
                    let date_split: Vec<&str> = date_str.split("-").collect();
                    let start_date_str = date_split[0];
                    start_date = EventScraperJcf::naive_date_from_str(&start_date_str, start_date);

                    end_date = start_date;
                    if date_split.len() > 1 {
                        let end_date_str = date_split[1];
                        // // NOTE: workaround for year not in end_date
                        // let end_date_str = start_date.format("%Y/").to_string() + end_date_str;
                        end_date = EventScraperJcf::naive_date_from_str(&end_date_str, start_date);
                    }
                }
            }

            // not or not well structured event
            if start_date == NaiveDate::from_ymd_opt(0, 1, 1).unwrap() ||
                end_date == NaiveDate::from_ymd_opt(0, 1, 2).unwrap() ||
               name == "" {
                continue;
            }

            let open_time = "unknown".to_string();
            let organizer = "Japan Chess Federation".to_string();
            let revenue = "unknown".to_string();
            let fee = "unknown".to_string();
            let e = EventInfo {
                name,
                organizer,
                start_date,
                end_date,
                open_time,
                revenue,
                fee,
            };
            events.push(e);
        }

        events
    }
}

impl EventScraperJcf {
    fn naive_date_from_str(input: &str, start_date: NaiveDate) -> NaiveDate {
        // input example: "2024/1/20(土)", "1/8(月祝)"
        // Return 1995-10-5 when failed to parase
        // When either day, month, year is missing, use start_date

        // println!("input: {:?}", input);
        // normalize input as much as possible (e.g., Zenkaku)
        let input_nfkd = input.nfkd().collect::<String>();
        // println!("input_nfkd: {:?}", input_nfkd);

        let only_ymd = trim_left(&input_nfkd, Vec::from([String::from("(")]));

        let mut slash_split_list: Vec<&str> = only_ymd.split("/").collect();
        // slash_split_list should be either: (%d-%m-%Y), (%d-%m), (%d), ()
        // println!("slash_split_list: {:?}", slash_split_list);

        let day_str = match slash_split_list.pop() {
            Some(val) => val.to_string(),
            None => start_date.format("%d").to_string(),
        };
        let day_int: u32 = day_str.parse().unwrap_or(5);
        // println!("day_str: {:?}", day_str);


        let month_str = match slash_split_list.pop() {
            Some(val) => val.to_string(),
            None => start_date.format("%m").to_string(),
        };
        let month_int: u32 = month_str.parse().unwrap_or(10);

        let year_str = match slash_split_list.pop() {
            Some(val) => val.to_string(),
            None => start_date.format("%Y").to_string(),
        };
        let year_int: i32 = year_str.parse().unwrap_or(1995);

        let datetime = NaiveDate::from_ymd_opt(year_int, month_int, day_int);

        // FIXME don't use unwrap()
        let ret = datetime.unwrap();
        // println!("ret: {:?}", ret);
        ret
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
    let ret = save_events_to_db(&events);
    match ret {
        Ok(_) => println!("DB save OK"),
        Err(e) => println!("DB save Error: {:?}", e),
    }
}

fn print_event_list(title: &str, events: &Vec<EventInfo>) {
    println!("============ {:?} ============", title);
    for e in events {
        println!("event");
        println!("  - name: {:?}", e.name);
        println!("  - organizer: {:?}", e.organizer);
        println!("  - start_date: {:?}", e.start_date);
        println!("  - end_date: {:?}", e.end_date);
        println!("  - open_time: {:?}", e.open_time);
        println!("  - revenue: {:?}", e.revenue);
        println!("  - fee: {:?}", e.fee);
    }
    println!("");
}

fn save_events_to_db(events: &Vec<EventInfo>) -> Result<(), &'static str> {
    let db_user = env::var("DB_USER").unwrap_or("DB_USER not set".to_string());
    let db_password = env::var("DB_PASSWORD").unwrap_or("DB_PASSWORD not set".to_string());
    let db_name = env::var("DB_NAME").unwrap_or("DB_NAME not set".to_string());
    let db_socket = env::var("DB_SOCKET").ok();

    // TODO: refactor this error handling shit code
    if db_user == "DB_USER not set" ||
       db_password == "DB_PASSWORD not set" ||
       db_name == "DB_NAME not set" {
           return Err("DB param environ not set")
    }


    let opts = if let Some(socket) = db_socket {
        OptsBuilder::default()
            .user(Some(db_user))
            .pass(Some(db_password))
            .db_name(Some(db_name))
            .socket(Some(socket))
    } else {
        // TODO add error handling for these environ
        let db_host = env::var("DB_HOST").expect("DB_HOST must be set");
        let db_port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
        OptsBuilder::default()
            .ip_or_hostname(Some(db_host))
            .user(Some(db_user))
            .pass(Some(db_password))
            .db_name(Some(db_name))
            .tcp_port(db_port.parse().expect("DB_PORT must be a valid number"))
    };

    // let pool = Pool::new(opts).unwrap_or(return Err("DB opt parse failed"));
    // let mut conn = pool.get_conn().unwrap_or(return Err("DB connect failed"));
    let pool = Pool::new(opts).unwrap();
    let mut conn = pool.get_conn().unwrap();

    for event in events {
        let ret = conn.exec_drop(
            r"INSERT INTO chess_event (name, organizer, start_date, end_date, open_time, revenue, fee)
              VALUES (:name, :organizer, :start_date, :end_date, :open_time, :revenue, :fee)",
            params! {
                "name" => event.name.to_string(),
                "organizer" => event.organizer.to_string(),
                "start_date" => event.start_date.format("%Y/%m/%d").to_string(),
                "end_date" => event.end_date.format("%Y/%m/%d").to_string(),
                "open_time" => event.open_time.to_string(),
                "revenue" => event.revenue.to_string(),
                "fee" => event.fee.to_string(),
            },
        );
        // TODO: return Err when failed
        match ret {
            Ok(_) => (),
            Err(e) => println!("Failed to insert event insert into DB: {:?}", e),
        };
    }
    Ok(())
}

fn fileoutput_event_list(title: &str, events: &Vec<EventInfo>, filepath: &str) {
    for e in events {
        let yaml_string = serde_yaml::to_string(&e).unwrap();

        println!("event yaml:");
        println!("{:?}", yaml_string);
    }
    println!("");
}
