use std::{
    fs::OpenOptions,
    io::{Read, Seek, Write},
};

use crate::Result;

pub struct Timestamp {
    time: i64,
    file: std::fs::File,
    updated: bool,
}

impl Timestamp {
    /// # Errors
    ///
    /// [`OpenOptions`]
    ///
    /// # Panics
    ///
    /// When temp file could not be read.
    pub fn load() -> Result<Self> {
        let path = format!(
            "{}.rss-notify",
            std::env::temp_dir()
                .to_str()
                .expect("could not get temp dir")
        );
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .read(true)
            .open(path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("could not read timestamp file");
        let time = contents.trim().parse::<i64>().unwrap_or_default();
        Ok(Self {
            time,
            file,
            updated: false,
        })
    }

    /// Write inner timestamp value into a temp file.
    ///
    /// # Errors
    ///
    /// Could fail while truncating/writing.
    pub fn write(&mut self) -> Result<()> {
        if self.updated {
            self.file.set_len(0)?;
            self.file.seek(std::io::SeekFrom::Start(0))?;
            self.updated = false;
            Ok(write!(self.file, "{}", self.time)?)
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.time
    }

    /// Update inner timestamp value.
    pub fn update(&mut self, ts: i64) {
        if ts > self.time {
            self.time = ts;
            self.updated = true;
        }
    }
}
