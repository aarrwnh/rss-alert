// #![allow(dead_code, unused_imports)]

use std::fmt::Write as _;
use std::io::Write as _;
use std::rc::Rc;
use std::thread::sleep;
use std::{collections::HashSet, time::Duration};

use chrono::{DateTime, Utc};

use rss_alert::{Result, config, item};

/// Hardcoded date offest cutout.
static PUBDATE_OFFSET: i64 = 60 * 60 * 12;

fn main() -> Result<()> {
    let config = config::Config::default();

    let Ok(mut feeds) = config.parse_feeds() else {
        panic!("something went wrong when parsing feeds")
    };

    println!("# TOAST_INTERVAL: {}s", config.toast_duration.as_secs());
    println!("# REFRESH_INTERVAL: {}s", config.cycle_interval.as_secs());

    for feed in &mut feeds {
        feed.url = urlencoding::decode(&feed.url).expect("UTF-8").to_string();
        println!("#  {}", feed.wrap_color(&feed.url)?);
    }
    println!();

    let mut entries = 0;
    let mut cache = HashSet::new();
    let mut out = String::new();
    let mut stdout = std::io::stdout();
    let interval = Duration::from_millis(100);

    loop {
        let n = Utc::now();
        let cutoff = n.timestamp() - PUBDATE_OFFSET;

        for feed in &feeds {
            let url = &feed.url;

            let items = match item::fetch_items(url) {
                Ok(x) => {
                    write!(&stdout, ".")?;
                    x
                }
                Err(err) => {
                    // https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
                    // out.write_str(&format!("\x1b]8;;{url}\x1b\\.\x1b]8;;\x1b[31m -< {err}\x1b[0m\n"))?;
                    let err = urlencoding::decode(&err.to_string())
                        .expect("UTF-8")
                        .to_string();

                    // probably just a network error
                    let msg = if err.contains(url) {
                        format!("\x1b[31m{err}\x1b[0m\n")
                    } else {
                        format!("{url}\x1b[31m -< {err}\x1b[0m\n")
                    };
                    out.write_str(&msg)?;
                    write!(&stdout, "\x1b[31m.\x1b[0m")?;
                    continue;
                }
            };

            sleep(interval);
            stdout.flush()?;

            let can_toast = !feed.no_toast();

            for el in items.iter().filter(|x| x.timestamp() > cutoff) {
                if cache.insert(Rc::clone(el)) && entries > 0 {
                    let pub_date = DateTime::from_timestamp(el.timestamp(), 0)
                        .map(|dt| dt.format("%H:%M"))
                        .expect("item publication date");
                    let line = format!("{pub_date} | {} | {}\n", el.link(), el.title());
                    let line = feed.wrap_color(line)?;
                    out.write_str(&line)?;
                    if can_toast {
                        el.show_toast(config.toast_duration);
                    }
                }
            }
        }

        write!(&stdout, "end\x1b[1G\x1b[2K{out}")?;
        stdout.flush()?;
        out.clear();

        cache.retain(|x| x.timestamp() > cutoff);
        entries = cache.len();

        sleep(config.cycle_interval);
    }
}
