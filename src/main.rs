// #![allow(dead_code, unused_imports)]

use std::fmt::Write as _;
use std::io::Write as _;
use std::rc::Rc;
use std::thread::sleep;
use std::{collections::HashSet, time::Duration};

use chrono::Utc;

use rss_alert::{Result, Timestamp, config, item};

fn main() -> Result<()> {
    let config = config::Config::default();

    let Ok(mut feeds) = config.parse_feeds() else {
        panic!("something went wrong when parsing feeds")
    };

    println!("# TOAST_INTERVAL: {}s", config.toast_duration.as_secs());
    println!("# REFRESH_INTERVAL: {}s", config.cycle_interval.as_secs());

    for feed in &mut feeds {
        feed.url = urlencoding::decode(&feed.url).expect("UTF-8").to_string();
        println!("#  {}", feed);
    }
    println!();

    let mut cache = HashSet::new();
    let mut out = String::new();
    let mut stdout = std::io::stdout();
    let interval = Duration::from_millis(100);
    let mut prev_update = Utc::now();
    let td = chrono::TimeDelta::new(config.cycle_interval.as_secs() as i64 * 3, 0).unwrap();

    let mut cutoff = Timestamp::load()?;
    let mut cutoff0 = 0;

    loop {
        let n = Utc::now();
        let diff = n - prev_update;
        prev_update = n;

        // XXX: skip once in case loop starts relatively fast after pc wake up?
        if diff > td {
            println!("-------");
            sleep(config.cycle_interval);
            continue;
        }

        let prev_cutoff = cutoff.timestamp();

        for feed in &feeds {
            let url = &feed.url;

            let items = match item::fetch_items(url) {
                Ok(x) => {
                    write!(&stdout, ".")?;
                    x
                }
                Err(err) => {
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

            let can_toast = feed.can_toast();

            for el in items
                .iter()
                .filter(|x| x.timestamp() > prev_cutoff)
                .filter(|x| cache.insert(Rc::clone(x)))
            {
                let ts = el.timestamp();
                cutoff0 = cutoff0.max(ts);

                // prevent toast on first run
                if cutoff.timestamp() == 0 {
                    continue;
                }

                let pub_date = chrono::DateTime::from_timestamp(ts, 0)
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

        write!(&stdout, "end\x1b[1G\x1b[2K{out}")?;
        stdout.flush()?;
        out.clear();

        cache.retain(|x| x.timestamp() > prev_cutoff);
        cutoff.save(cutoff0)?;

        sleep(config.cycle_interval);
    }
}
