#[macro_use]
pub mod list;

use lazy_static::lazy_static;
use regex_lite::Regex;
use std::collections::HashMap;
use tracing::{error, instrument};

pub struct Shorthand {
    pattern: Regex,
    format: String,
}

lazy_static! {
    static ref FORMAT_PATTERN: Regex = Regex::new(r"\$([\d])+").unwrap();
}

impl Shorthand {
    fn find_arg_count_in_format_str(format: &str) -> Option<usize> {
        let mut max = 0;

        // Debug capture & match output
        // FORMAT_PATTERN.captures_iter(format).for_each(|x| println!("{}", x.iter().map(|y| y.unwrap().as_str()).collect::<Vec<_>>().join(" :: ")));

        for cap in FORMAT_PATTERN.captures_iter(format) {
            let num = cap.get(1).and_then(|x| x.as_str().parse::<usize>().ok())?;
            if num > max {
                max = num;
            }
        }
        Some(max)
    }

    #[instrument]
    pub fn try_new(pattern: &str, format: &str) -> Option<Self> {
        let regex = Regex::new(pattern).ok()?;
        if regex.captures_len() <= Self::find_arg_count_in_format_str(format)? {
            error!("Attempted to create shorthand which references invalid captures");
            return None;
        }

        Some(Self {
            pattern: regex,
            format: format.to_string(),
        })
    }

    pub fn apply_to_string(&self, target: &str) -> Option<String> {
        let matches_start = self.pattern.is_match_at(target, 0);
        if !matches_start {
            return None;
        }

        let cap = self
            .pattern
            .captures(target)
            .expect("Matching already checked");
        let mut capture_map = HashMap::new();
        for (i, m) in cap.iter().enumerate() {
            if let Some(m) = m {
                capture_map.insert(i, m.as_str());
            } else {
                capture_map.insert(i, "");
            }
        }

        let mut format = self.format.clone();

        // TODO: go over this with the format arg regex, in case of trailing characters
        for (idx, repl) in capture_map {
            format = format.replace(&format!("${idx}"), repl);
        }

        Some(format)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn shorthand_t1() {
        let sh = Shorthand::try_new(r"f([\.]?)(\d)", "$1$2").unwrap();
        assert_eq!(".6", sh.apply_to_string("f.6").unwrap());
        assert_eq!("6", sh.apply_to_string("f6").unwrap());
    }
}
