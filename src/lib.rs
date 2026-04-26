use std::sync::LazyLock;

use tauri_winrt_notification::{IconCrop, Toast};

pub mod item;
pub use item::{Element, fetch_items};

mod config;
pub use config::{Config, Feed};

mod error;
pub use error::Result;

mod timestamp;
pub use timestamp::Timestamp;

static ICON_PATH: LazyLock<std::path::PathBuf> =
    LazyLock::new(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("rss.png"));

// ----------------------------------------------------------------------------------
//   - Toaster -
// ----------------------------------------------------------------------------------
pub trait Toastable: std::fmt::Debug {
    fn title(&self) -> &str;
    fn link(&self) -> &str;
    fn timestamp(&self) -> i64;
    fn extra(&self) -> Option<&str>;

    fn show_toast(&self, wait_sec: std::time::Duration) {
        Toast::new(Toast::POWERSHELL_APP_ID)
            .title(self.title())
            .text1(self.link())
            .icon(&ICON_PATH, IconCrop::Square, "rss")
            .sound(None)
            .show()
            .expect("unable to show toast notification");
        std::thread::sleep(wait_sec);
    }
}
