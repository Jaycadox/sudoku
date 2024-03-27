use crate::status_bar::shorthands::Shorthand;

#[derive(Default)]
pub struct List {
    shorthands: Vec<Shorthand>,
}

impl List {
    pub fn add(mut self, pattern: &str, format: &str) -> Option<Self> {
        let sh = Shorthand::try_new(pattern, format)?;
        self.shorthands.push(sh);
        Some(self)
    }

    pub fn apply_to_string(&self, target: &str) -> Option<String> {
        self.shorthands
            .iter()
            .find_map(|x| x.apply_to_string(target))
    }
}

#[macro_export]
macro_rules! shorthand {
    ($(($pattern:expr, $replacement:expr)),* $(,)?) => {{
        let shorthand_list = Some(List::default());
        $(
            let shorthand_list = shorthand_list.and_then(|x| x.add($pattern, $replacement));
        )*
        shorthand_list
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_valid_regex_and_format() {
        let _ = shorthand![
            (r"f([\.]?)(\d)", "$1$2"),
            (r"pm(\d)", "$1"),
            (r"test(\d+)", "$1"),
            (r"abc(def)", "$1"),
            (r"xyz", "$0"),
        ]
        .unwrap();
    }

    #[test]
    fn test_invalid_regex_formatting() {
        assert!(shorthand![
            (r"invalid[", "$1$2"),
            (r"invalid*", "$1$2"),
            (r"invalid(", "$1$2"),
            (r"invalid{", "$1$2"),
            (r"invalid^", "$1$2"),
        ]
        .is_none())
    }

    #[test]
    fn test_invalid_format_string() {
        assert!(shorthand![
            (r"f([\.]?)(\d)", "$9$2"),
            (r"pm(\d)", "$1$3"),
            (r"test(\d+)", "$1$2$3"),
            (r"abc(def)", "$1$2"),
            (r"xyz", "$2"),
        ]
        .is_none())
    }

    #[test]
    fn test_invalid_regex_and_format_string() {
        assert!(shorthand![
            (r"invalid[", "$3$2"),
            (r"invalid*", "$1$2"),
            (r"invalid(", "$1$2"),
            (r"invalid{", "$1$2"),
            (r"invalid^", "$1$2"),
        ]
        .is_none())
    }

    #[test]
    fn test_invalid_capture_group_reference() {
        assert!(shorthand![
            (r"f([\.]?)(\d)", "$3$2"),
            (r"pm(\d)", "$1$3"),
            (r"test(\d+)", "$1$2$3"),
            (r"abc(def)", "$1$2"),
            (r"xyz", "$2"),
        ]
        .is_none())
    }
}
