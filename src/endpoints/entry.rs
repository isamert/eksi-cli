use std::collections::HashMap;
use select::node::Node;
use select::predicate::{Class, Name};

// self
use endpoints::author::Author;

// TODO: add last_edit (parse from date)
#[derive(Debug)]
pub struct Entry {
    pub id: i32,
    pub author: Author,
    pub fav_count: String,
    pub is_fav: bool,
    pub text: String,
    pub date: String,
}

impl Entry {
    // TODO: handle sanitizer mode
    // TODO: find a way to do it internally
    pub fn sanitized(node: &Node) -> String {
        let mut text = node.inner_html();

        // bkz
        for x in node.find(Class("b")) {
            text = text.replace(x.html().trim(), x.text().trim());
        }

        // gizli bkz
        for x in node.find(Class("ab")) {
            let gbkz = x.find(Name("a")).next().unwrap().attr("data-query").unwrap().trim();
            text = text.replace(x.html().trim(), gbkz);
        }

        // links
        // TODO: <a>http://linkin-kendisi</a>
        let mut links = vec![];
        for (i, x) in node.find(Class("url")).enumerate() {
            let url_text = x.text().trim().to_string()
                + "[" + &i.to_string() + "]";
            text = text.replace(x.html().trim(), &url_text);
            links.push(x.attr("href").unwrap_or("it goes nowhere"));
        }

        let links_text: String = links.into_iter()
            .enumerate()
            .map(|(i, x)| format!("\n[{}]: {}", i, x))
            .collect();

        text.replace("<br/>", "\n")
            .replace("<br />", "\n")
            .replace("<br>", "\n")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .trim()
            .to_string()
            + &links_text
    }
}
