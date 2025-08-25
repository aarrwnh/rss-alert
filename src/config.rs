use crate::Result;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Write as _},
    io::Read as _,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

pub struct Config {
    /// Path pointing to file containing list of feeds
    pub file_path: PathBuf,
    /// Delay amount of seconds between each `Toast::new` call
    /// TODO? this should grab value from somewhere in the registry
    pub toast_duration: Duration,
    /// Interval between each update cycle
    pub cycle_interval: Duration,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self::new(600, 5)
    }
}

impl Config {
    #[inline]
    #[must_use]
    /// # Panics
    ///
    /// it will
    pub fn new(cycle_interval: u64, toast_duration: u64) -> Self {
        fn parse_number(default: u64, x: Option<&String>) -> Duration {
            Duration::from_secs(x.map_or(default, |v| {
                u64::from_str(v).expect("tried to parse a number")
            }))
        }

        let args = Self::get_args();
        let file_path = PathBuf::from(args.get("--path").expect("--path is required"));

        if !file_path.exists() {
            eprintln!("{} does not exists", file_path.display());
            std::process::exit(0);
        }

        Self {
            file_path,
            toast_duration: parse_number(toast_duration, args.get("--toast")),
            cycle_interval: parse_number(cycle_interval, args.get("--interval")),
        }
    }

    fn get_args() -> HashMap<String, String> {
        std::env::args()
            .skip(1)
            .filter_map(|s| s.split_once('=').map(|(a, b)| (a.to_owned(), b.to_owned())))
            .collect()
    }

    /// # Errors
    ///
    /// This function will return an error if config file is not found.
    pub fn parse_feeds(&self) -> Result<Vec<Feed>> {
        let buf = read_file(&self.file_path)?;
        Ok(parse_feeds_var(&buf))
    }
}

#[inline]
fn read_file(path: impl AsRef<Path>) -> Result<String> {
    let mut file = std::fs::File::open(path.as_ref())?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf.trim().to_owned())
}

// s = "var1={1|2|3}&var2={4|5}"
// result = [ f"var1={v1}&var2{v2}" for v1 in [1,2,3] for v2 in [4,5] ]
fn parse_feeds_var(s: &str) -> Vec<Feed> {
    let mut feeds = Vec::new();
    let mut temp = Vec::new();
    let mut args = BTreeMap::new();

    let mut options = HashMap::new();

    for line in s.split('\n') {
        if line.trim().starts_with('#') {
            continue;
        }

        let mut index = 0;
        let end = line.len();

        while index < end {
            let mut rest = &line[index..end];
            // parse some options
            if index == 0 && rest.find('[').map(|x| x <= 2).is_some_and(|x| x) {
                if let Some(j) = rest[0..].find(']') {
                    rest[1..j].split(' ').for_each(|x| {
                        if let Some((key, value)) = x.split_once('=')
                            && ALLOWED_KEYS.contains(&key)
                        {
                            if let Ok(value) = Value::from_str(value) {
                                options.insert(key.to_string(), value);
                            }
                        }
                    });
                    index += j + 2;
                    rest = &rest[index..];
                }
            }

            // parse url
            if let Some(mut p) = rest.find('{') {
                temp.push(&rest[0..p]);
                if let Some(j) = rest[p..].find('}') {
                    let v = rest[p + 1..p + j].split('|').collect();
                    args.insert(temp.len(), v); // key=index in template
                    temp.push(""); // empty position for `replacement`
                    p += j;
                }
                index += p + 1;
            } else {
                temp.push(rest);
                index = end;
            }
        }

        if args.is_empty() {
            assert_eq!(temp.len(), 1);
            feeds.push(Feed {
                url: temp[0].to_owned(),
                options: options.clone(),
            });
        } else {
            let entries = args.keys().collect::<Vec<_>>();
            let arrays = args.values().collect::<Vec<_>>();
            for r in combinations(&arrays) {
                for (pos, replacement) in r.iter().enumerate() {
                    temp[*entries[pos]] = replacement;
                }
                feeds.push(Feed {
                    url: temp.join(""),
                    options: options.clone(),
                });
            }
        }

        temp.clear();
        args.clear();
        options.clear();
    }
    feeds
}

fn get_combinations<T: Copy + Debug>(n: usize, arrays: &[&Vec<T>], divisors: &[usize]) -> Vec<T> {
    arrays
        .iter()
        .enumerate()
        .map(|(i, arr)| arr[(n / divisors[i]) % arr.len()])
        .collect()
}

fn combinations<T: Copy + Debug>(arrays: &[&Vec<T>]) -> Vec<Vec<T>> {
    let mut divisors = vec![0; arrays.len()];
    let mut count = 1;

    for i in (0..arrays.len()).rev() {
        divisors[i] = match divisors.get(i + 1) {
            Some(v) => v * arrays[i + 1].len(),
            None => 1,
        };
        match arrays[i].len() {
            0 => (),
            x => count *= x,
        }
    }

    (0..count)
        .map(|i| get_combinations(i, arrays, &divisors))
        .collect()
}

// ----------------------------------------------------------------------------------
//   - Feed -
// ----------------------------------------------------------------------------------
const ALLOWED_KEYS: &[&str] = &["foreground", "background", "no_toast"];

#[derive(Debug, Clone)]
pub struct Feed {
    pub url: String,
    pub options: HashMap<String, Value>,
}

impl Feed {
    const COLOR_OPTS: [(&'static str, &'static str); 2] =
        [("foreground", "38"), ("background", "48")];

    /// Put colors into ANSI escape sequence.
    pub fn wrap_color(&self, text: impl AsRef<str>) -> Result<String> {
        let mut res = String::new();
        for (key, attr) in Self::COLOR_OPTS {
            if let Some(color) = self.options.get(key) {
                write!(res, "\x1b[{attr};5;{color}m")?;
            }
        }
        let with_escape = !res.is_empty();
        write!(res, "{}", text.as_ref())?;
        if with_escape {
            write!(res, "\x1b[m")?;
        }
        Ok(res)
    }

    /// Check if we can show toast or just log text in console.
    #[must_use]
    pub fn no_toast(&self) -> bool {
        match self.options.get("no_toast") {
            Some(Value::Bool(b)) => *b,
            _ => false,
        }
    }
}

// ----------------------------------------------------------------------------------
//   - Value -
// ----------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    Bool(bool),
    String(String),
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        if let Ok(v) = s.parse::<bool>() {
            return Self::Bool(v);
        }
        if let Ok(v) = s.parse::<i32>() {
            return Self::Int(v);
        }
        Self::String(s.to_string())
    }
}

impl FromStr for Value {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.is_empty() {
            Err("empty value".to_string())
        } else {
            Ok(s.into())
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match *self {
            Self::Int(i) => i.to_string(),
            Self::Bool(b) => if b { "true" } else { "false" }.to_string(),
            Self::String(ref s) => s.to_string(),
        };
        write!(f, "{c}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl PartialEq<&str> for Feed {
        fn eq(&self, other: &&str) -> bool {
            self.url.eq(other)
        }
    }

    #[test]
    fn expand_vars() {
        let a = parse_feeds_var("{A|B|C}_{D|E}\nasdf\n# {DD|CC}");
        assert_eq!(a, vec!["A_D", "A_E", "B_D", "B_E", "C_D", "C_E", "asdf"]);
    }

    #[test]
    fn expand_vars_and_options() {
        let a = parse_feeds_var(
            "[foreground=123 backgroun=321 no_toast=true] {A|B|C}_{D|E}\nasdf\n# {DD|CC}",
        );
        assert_eq!(a, vec!["A_D", "A_E", "B_D", "B_E", "C_D", "C_E", "asdf"]);
        let mut o = HashMap::new();
        o.insert("foreground".into(), "123".into());
        o.insert("no_toast".into(), "true".into());
        assert_eq!(a[0].options, o);
    }
}
