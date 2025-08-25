use std::{fmt::Debug, hash::Hash, rc::Rc, str::FromStr, time::Duration};

use chrono::DateTime;
use roxmltree::Node;

use crate::{Result, Toastable};

pub fn get_rss_feed(endpoint: &str) -> Result<String> {
    Ok(reqwest::blocking::get(endpoint)?.text()?)
}

pub fn fetch_items(endpoint: &str) -> Result<Vec<Rc<Element>>> {
    let body = get_rss_feed(endpoint)?;
    let doc = match roxmltree::Document::parse(&body) {
        Ok(doc) => doc,
        Err(err) => {
            return Err(format!("parsed body is not a valid XML string: {err}").into());
        }
    };

    Ok(doc
        .descendants()
        .filter_map(|n| {
            let item = n.children().filter_map(|n| {
                let Ok(tag) = Tag::from_str(n.tag_name().name()) else {
                    return None;
                };
                Some((tag, n))
            });
            let element = match n.tag_name().name() {
                "item" => Element::Item(item.collect()),
                "entry" => Element::Entry(item.collect()),
                _ => return None,
            };
            Some(Rc::new(element))
        })
        .rev()
        .collect())
}

// ----------------------------------------------------------------------------------
//   - Item -
// ----------------------------------------------------------------------------------
macro_rules! item {
    ($name:ident, $f:expr) => {
        #[derive(Default, Clone, Debug, Eq)]
        pub struct $name {
            title: String,
            link: String,
            /// UTC timestamp
            pub_date: i64,
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                // should be fine as long date is parsed...
                self.pub_date == other.pub_date
            }
        }

        impl Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.pub_date.hash(state);
            }
        }

        impl Toastable for $name {
            fn get_title(&self) -> &str {
                &self.title
            }
            fn get_link(&self) -> &str {
                &self.link
            }
            fn get_timestamp(&self) -> i64 {
                self.pub_date
            }
        }

        impl<'a: 'b, 'b> FromIterator<(Tag, Node<'a, 'b>)> for $name {
            fn from_iter<T: IntoIterator<Item = (Tag, Node<'a, 'b>)>>(iter: T) -> Self {
                let mut item = $name::default();
                for (tag, node) in iter {
                    let text = node.text().unwrap_or_default().to_owned();
                    $f(&mut item, tag, text, node);
                }
                item
            }
        }
    };
}

// RSS2.0 spec
// https://www.rssboard.org/rss-specification#hrelementsOfLtitemgt
item!(Item, |item: &mut Item, tag, text, _| {
    match tag {
        Tag::Title => item.title = text,
        Tag::Guid => item.link = text,
        Tag::Date => match DateTime::parse_from_rfc2822(&text) {
            Ok(dt) => item.pub_date = dt.timestamp(),
            Err(e) => todo!("{}", e),
        },
        _ => {}
    }
});

// custom spec?
item!(Entry, |item: &mut Entry, tag, text, node: Node<'_, '_>| {
    match tag {
        Tag::Title => item.title = text,
        Tag::Link => {
            item.link = match node.attribute("href") {
                Some(href) => href.to_owned(),
                None => text,
            }
        }
        Tag::Updated => match DateTime::parse_from_rfc3339(&text) {
            Ok(dt) => item.pub_date = dt.timestamp(),
            Err(e) => todo!("{}", e),
        },
        _ => {}
    }
});

// ----------------------------------------------------------------------------------
//   - Tag -
// ----------------------------------------------------------------------------------
enum Tag {
    Title,
    Link,
    Date,
    Updated,
    Guid,
}

impl FromStr for Tag {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "title" => Tag::Title,
            "link" => Tag::Link,
            "guid" => Tag::Guid,
            "pubDate" => Tag::Date,    // Mon, 13 Jan 2025 15:04:31 -0000
            "updated" => Tag::Updated, // 2025-01-13T00:00:00+09:00
            _ => return Err("xml tag not supported"),
        })
    }
}

// ----------------------------------------------------------------------------------
//   - Element -
// ----------------------------------------------------------------------------------
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Element {
    Item(Item),
    Entry(Entry),
}

impl Element {
    pub fn inner(&self) -> &dyn Toastable {
        match self {
            Element::Item(item) => item,
            Element::Entry(entry) => entry,
        }
    }

    pub fn timestamp(&self) -> i64 {
        self.inner().get_timestamp()
    }

    pub fn title(&self) -> &str {
        self.inner().get_title()
    }

    pub fn link(&self) -> &str {
        self.inner().get_link()
    }

    pub fn show_toast(&self, wait_sec: Duration) {
        self.inner().show_toast(wait_sec);
    }
}
