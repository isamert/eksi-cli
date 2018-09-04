use std::fmt;
use reqwest;
use select::document::Document;

use eksi;
use extensions::UrlConvertable;
use endpoints::entry::Entry;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Title {
    pub id: i32,
    pub title: String,
    pub popular_count: Option<String>,
}

impl Title {
    pub fn to_url(&self, page: usize, popular: bool) -> String {
         "https://eksisozluk.com".to_string() + "/" +  &self.title
            .to_lowercase()
            .replace(" ", "-")
            .to_url()
            + &("--".to_string() + &self.id.to_string())
            + &("?p=".to_string() + &(page + 1).to_string())
            + if popular { "&a=popular" } else { "" }
     }

    pub fn entries(&self, page: usize, popular: bool) -> Vec<Entry> {
        let text = reqwest::get(&self.to_url(page, popular))
                    .unwrap()
                    .text()
                    .unwrap();
        let doc = Document::from(&text[..]);

        eksi::entries_of(&doc, popular)
    }
}

impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.popular_count {
            Some(ref a) => write!(f, "{} ({})", self.title, a),
            None        => write!(f, "{}", self.title)
        }
    }
}
