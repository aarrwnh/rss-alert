use std::fmt::Write as _;
use std::io::Write as _;
use std::rc::Rc;
use std::thread::sleep;
use std::{collections::HashSet, time::Duration};

use chrono::{TimeDelta, Utc};

use rss_alert::{Config, Element, Feed, Result, Timestamp, fetch_items};

fn main() -> Result<()> {
    if let Ok(mut app) = App::new(100) {
        app.run()?;
    }
    Ok(())
}

struct App {
    cache: HashSet<Rc<Element>>,
    config: Config,
    /// Temporary holder for the cutoff value.
    cutoff: Timestamp,
    /// Some interval between consecutive http calls when there's 0 toasts shown.
    interval: Duration,
    out: String,
    stdout: std::io::Stdout,
    td: TimeDelta,
}

impl App {
    fn new(interval: u64) -> Result<Self> {
        let config = Config::default();
        #[allow(clippy::cast_possible_wrap)]
        let td = TimeDelta::new(config.cycle_interval.as_secs() as i64 * 3, 0).unwrap();

        Ok(Self {
            cache: HashSet::new(),
            config,
            cutoff: Timestamp::load()?,
            interval: Duration::from_millis(interval),
            out: String::new(),
            stdout: std::io::stdout(),
            td,
        })
    }

    fn run(&mut self) -> Result<()> {
        let Ok(mut feeds) = self.config.parse_feeds() else {
            panic!("something went wrong when parsing feeds")
        };

        let toast_duration = self.config.toast_duration;
        let cycle_interval = self.config.cycle_interval;
        println!("# TOAST_INTERVAL: {}s", toast_duration.as_secs());
        println!("# REFRESH_INTERVAL: {}s", cycle_interval.as_secs());

        for feed in &mut feeds {
            feed.url = urlencoding::decode(&feed.url).expect("UTF-8").to_string();
            println!("#  {feed}");
        }
        println!();

        let mut prev_update = Utc::now();

        loop {
            let n = Utc::now();
            let diff = n - prev_update;
            prev_update = n;

            // XXX: skip once in case loop starts relatively fast after pc wake up?
            if diff > self.td {
                println!("-------");
                sleep(cycle_interval);
                continue;
            }

            for feed in &feeds {
                let Some(items) = self.fetch_items(&feed.url) else { continue };
                sleep(self.interval);
                self.stdout.flush()?;
                self.filter_items(feed, &items)?;
            }

            self.cleanup()?;
            sleep(self.config.cycle_interval);
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        write!(&self.stdout, "end\x1b[1G\x1b[2K{}", self.out)?;
        self.stdout.flush()?;
        self.out.clear();
        let prev_cutoff = self.cutoff.timestamp();
        self.cache.retain(|x| x.timestamp() > prev_cutoff);
        self.cutoff.write()?;
        Ok(())
    }

    fn fetch_items(&mut self, url: &str) -> Option<Vec<Rc<Element>>> {
        match fetch_items(url) {
            Ok(x) => {
                write!(&self.stdout, ".").ok()?;
                Some(x)
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
                self.out.write_str(&msg).ok()?;
                write!(&self.stdout, "\x1b[31m.\x1b[0m").ok()?;
                None
            }
        }
    }

    fn filter_items(&mut self, feed: &Feed, items: &[Rc<Element>]) -> Result<()> {
        let prev_cutoff = self.cutoff.timestamp();
        let can_toast = feed.can_toast();
        for el in items
            .iter()
            .filter(|x| x.timestamp() > prev_cutoff)
            .filter(|x| self.cache.insert(Rc::clone(x)))
        {
            let ts = el.timestamp();
            self.cutoff.update(ts);

            // prevent toast on first run
            if self.cutoff.timestamp() == 0 {
                continue;
            }

            let pub_date = chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%H:%M"))
                .expect("item publication date");

            let line = feed.wrap_color(format!(
                "{pub_date} | {} | {}{}\n",
                el.link(),
                el.title(),
                el.extra().unwrap_or_default()
            ))?;
            self.out.write_str(&line)?;

            if can_toast {
                el.show_toast(self.config.toast_duration);
            }
        }
        Ok(())
    }
}
