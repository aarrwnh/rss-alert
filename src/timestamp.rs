use std::{
    fs::OpenOptions,
    io::{Read, Seek, Write},
};

use crate::Result;

pub struct Timestamp(i64, std::fs::File);

impl Timestamp {
    /// # Errors
    ///
    /// [`OpenOptions`]
    ///
    /// # Panics
    ///
    /// When temp file could not be read.
    pub fn load() -> Result<Self> {
        let path = format!("{}.rss-notify", std::env::temp_dir().to_str().expect("could not get temp dir"));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .read(true)
            .open(path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("could not read timestamp file");
        let ts = contents.trim().parse::<i64>().unwrap_or_default();
        Ok(Self(ts, file))
    }

    /// Update and save timestamp into temp file.
    ///
    /// # Errors
    ///
    /// Could fail while truncating/writing.
    pub fn save(&mut self, ts: i64) -> Result<()> {
        if ts > self.0 {
            self.0 = ts;
            self.1.set_len(0)?;
            self.1.seek(std::io::SeekFrom::Start(0))?;
            Ok(write!(self.1, "{ts}")?)
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub const fn timestamp(&self) -> i64 {
        self.0
    }
}
