const CHESS_CLUB_LIST: [&str; 3] = ["8x8_chess_club", "kitasenjyu", "ncs"];

fn main() {
    println!("CHESS_CLUB_LIST: {:?}", CHESS_CLUB_LIST);

    // FIXME: unwrap()

    let club8x8_scraper = ChessEventScraperFactory::create("8x8_chess_club");
    let events = club8x8_scraper.unwrap().scrape_event();
    println!("8x8");
    for e in events {
        println!("================ event ==============");
        println!("date: {:?}", e.date);
        println!("open_time: {:?}", e.open_time);
        println!("revenue: {:?}", e.revenue);
        println!("fee: {:?}", e.fee);
    }

    let kitasenjyu_scraper = ChessEventScraperFactory::create("kitasenjyu");
    let events = kitasenjyu_scraper.unwrap().scrape_event();
    println!("kitasenjyu");
    for e in events {
        println!("================ event ==============");
        println!("date: {:?}", e.date);
        println!("open_time: {:?}", e.open_time);
        println!("revenue: {:?}", e.revenue);
        println!("fee: {:?}", e.fee);
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
        println!("======This is EventScraperKitasenjyu=======");
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
