const CHESS_CLUB_LIST: [&str; 3] = ["club8x8", "kitasenjyu", "ncs"];

fn main() {
    println!("CHESS_CLUB_LIST: {:?}", CHESS_CLUB_LIST);

    let club8x8_scraper = ChessEventScraperFactory::create("8x8_chess_club");

    let events = club8x8_scraper.scrape_event();
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
    fn create(keyword: &str) -> impl ChessEventScraper {
        // TODO add other than 8x8 club scraper
        match keyword {
            "8x8_chess_club" => EventScraperClub8x8 {},
            _ => panic!("Not supported keyword")  // FIXME Change to Result
        }
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
        let body = 
            reqwest::blocking::get(EventScraperClub8x8::url())
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

pub trait ChessClubScraper {
    fn to_yaml(&self) -> String;
    fn name(&self) -> &String;
    fn url(&self) -> &String;
    fn scrape_event(&self) -> Vec<EventInfo>;
}

// struct ChessClub8x8 {
//     _name: String,
//     _url: String,
// }

struct ChessClubKitaSenjyu {
    _name: String,
    _url: String,
}

impl ChessClubScraper for ChessClubKitaSenjyu {
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

        let scrape_target_selector = 
            scraper::Selector::parse("article.entry-content").unwrap();
        let scrape_target_list = document.select(&scrape_target_selector);
        let paragraph_selector = scraper::Selector::parse("p").unwrap();

        let mut events = Vec::new();
        for article in scrape_target_list {
            // log::info!("================");
            let mut date = String::from("");
            let mut open_time = String::from("");
            let mut revenue = String::from("");
            let mut fee = String::from("");
            for e in article.select(&paragraph_selector) {
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
