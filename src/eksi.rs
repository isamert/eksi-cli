use reqwest;
use select::node::Node;
use select::document::Document;
use select::predicate::{Attr, Class, Name};


// self
use eksi;
use endpoints::entry::Entry;
use endpoints::title::Title;
use endpoints::author::Author;

// FIXME: extract eksi url as constant
pub fn popular_titles(page: usize) -> Vec<Title> {
    let text = reqwest::get(&format!("https://eksisozluk.com/basliklar/gundem?p={}", page + 1))
            .unwrap()
            .text()
            .unwrap();

    let doc = Document::from(&text[..]);

                    // skip sol-frame
    eksi::titles_of(&doc.find(Attr("id", "content")).next().unwrap())
}

// FIXME: get Document, not Node
/// Returns a list of titles from given `Node`
pub fn titles_of(doc: &Node) -> Vec<Title> {
    let mut vec = Vec::new();

    let titles = doc.find(Class("topic-list"))
                    .next()
                    .unwrap()
                    .find(Name("li"));

    for node in titles {
        vec.push(Title {
            title: match node.find(Name("a")).next() {
                Some(a)  => a.children().next().unwrap().text().trim().to_string(),
                None     => continue // Skip if we dont have the title.
            },
            id: node.find(Name("a")).next().map(|x| eksi::id_of(x.attr("href").unwrap()).unwrap()).unwrap(),
            popular_count:  node.find(Name("small")).next().map(|x| x.text()),
        });
    }

    vec
}

// TODO: Ask for sanitizer mode
/// Returns the entry list from given `Document`
pub fn entries_of(doc: &Document, popular: bool) -> Vec<Entry> {
    let mut vec = Vec::new();

    let entries = doc.find(Attr("id", "entry-item-list"))
                     .next()
                     .unwrap()
                     .find(Name("li"));

    for node in entries {
        vec.push(Entry {
            id: match node.attr("data-id") {
                Some(a) => a.parse::<i32>().unwrap(),
                None    => continue // If this one exists, other ones surely will exist
            },
            author: Author {
                id: node.attr("data-author-id").unwrap().parse::<i32>().unwrap(),
                name: node.attr("data-author").unwrap().to_string(),
            },
            fav_count: node.attr("data-favorite-count").unwrap().parse::<i32>().unwrap().to_string(),
            is_fav: node.attr("data-isfavorite").unwrap().parse::<bool>().unwrap(),
            date: node.find(Class("entry-date")).next().unwrap().text(),
            text: Entry::sanitized(&node.find(Class("content")).next().unwrap()),
        });
    }

    vec
}

pub fn search(query: &str) -> Option<(Title, Vec<Entry>)> {
    let text = reqwest::get(&format!("https://eksisozluk.com/?q={}", query))
            .unwrap()
            .text()
            .unwrap();

    let doc = Document::from(&text[..]);
    let title_not_found = doc.find(Attr("id", "topic")).next().unwrap().attr("data-not-found");
    match title_not_found {
        Some("true") => { // The title doesn't exists
            None
        },
        _ => { // The title exists
            let title_node = doc.find(Attr("id", "title")).next().unwrap();
            let title = Title {
                id: title_node.attr("data-id").unwrap().parse().unwrap(),
                title: title_node.attr("data-title").unwrap().to_string(),
                popular_count: None
            };
            let entries = entries_of(&doc, false);

            Some((title, entries))
        }
    }
}

/// Returns the id of particular tite from given url
pub fn id_of(href: &str) -> Option<i32> {
    href.split("--")
        .collect::<Vec<_>>()[1]
        .chars()
        .take_while(|x| x.is_numeric())
        .collect::<String>()
        .parse()
        .ok()
}
