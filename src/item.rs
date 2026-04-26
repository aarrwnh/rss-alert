#![allow(dead_code)]

use std::{
    fmt::{Debug, Write},
    hash::Hash,
    ops::Deref,
    rc::Rc,
    str::FromStr,
};

use chrono::DateTime;
use roxmltree::Node;

use crate::{Result, Toastable};

pub fn get_rss_feed(endpoint: &str) -> Result<String> {
    Ok(reqwest::blocking::get(endpoint)?.text()?)
}

/// # Errors
pub fn fetch_items(endpoint: &str) -> Result<Vec<Rc<Element>>> {
    let body = get_rss_feed(endpoint)?;
    let doc = roxmltree::Document::parse(&body)
        .map_err(|err| format!("parsed body is not a valid XML string: {err}"))?;

    Ok(doc
        .descendants()
        .filter_map(|n| {
            let item = n
                .children()
                .filter_map(|n| match Tag::from_str(n.tag_name().name()) {
                    Ok(tag) => Some((tag, n)),
                    Err(_) => None,
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
            extra: Option<String>,
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
            fn title(&self) -> &str {
                &self.title
            }
            fn link(&self) -> &str {
                &self.link
            }
            fn timestamp(&self) -> i64 {
                self.pub_date
            }
            fn extra(&self) -> Option<&str> {
                self.extra.as_deref()
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
            Err(_) => (), // premature end of input
        },
        Tag::Extra => {
            let s = item.extra.get_or_insert(String::new());
            _ = write!(s, " | {text}");
        }
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
            Err(_) => (),
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
    Extra,
}

impl FromStr for Tag {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "title" => Self::Title,
            "link" => Self::Link,
            "guid" => Self::Guid,
            "pubDate" => Self::Date,    // Mon, 13 Jan 2025 15:04:31 -0000
            "updated" => Self::Updated, // 2025-01-13T00:00:00+09:00
            "size" => Self::Extra,
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

impl Deref for Element {
    type Target = dyn Toastable;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Item(item) => item,
            Self::Entry(entry) => entry,
        }
    }
}
