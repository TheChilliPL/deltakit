use bon::__::IsUnset;
use bon::Builder;
use regex::{regex, Regex};
use crate::kvparser::k_v_parser_builder::{SetWithScopePatterns, State};

macro_rules! assert_static_capture_groups {
    ($regex:expr, 0) => {
        assert_eq!($regex.static_captures_len(), Some(1), "regex should not have capture groups");
    };
    ($regex:expr, 1) => {
        assert_eq!($regex.static_captures_len(), Some(2), "regex should have exactly one capture group");
    };
    ($regex:expr, $expected:expr) => {
        assert_eq!($regex.static_captures_len(), Some($expected + 1), "regex should have exactly {} capture groups", $expected);
    };
}

pub struct RegexPair<'a> {
    start: &'a Regex,
    end: &'a Regex,
}

impl <'a> RegexPair<'a> {
    pub fn new(start: &'a Regex, end: &'a Regex) -> Self {
        assert_static_capture_groups!(start, 0);
        assert_static_capture_groups!(end, 0);
        Self { start, end }
    }

    pub fn find_match_loc_in_at(&self, haystack: &str, start: usize) -> Option<(usize, usize)> {
        let start_mat = self.start.find_at(haystack, start)?;
        let start_in = start_mat.end();
        let end_mat = self.end.find_at(haystack, start_in)?;
        let end_in = end_mat.start();
        Some((start_in, end_in))
    }

    pub fn find_match_loc_out_at(&self, haystack: &str, start: usize) -> Option<(usize, usize)> {
        let start_mat = self.start.find_at(haystack, start)?;
        let start_out = start_mat.start();
        let start_in = start_mat.end();
        let end_mat = self.end.find_at(haystack, start_in)?;
        let end_out = end_mat.end();
        Some((start_out, end_out))
    }

    pub fn find_match_loc_in(&self, haystack: &str) -> Option<(usize, usize)> {
        self.find_match_loc_in_at(haystack, 0)
    }

    pub fn find_match_loc_out(&self, haystack: &str) -> Option<(usize, usize)> {
        self.find_match_loc_out_at(haystack, 0)
    }

    pub fn find_match_in_at<'b>(&self, haystack: &'b str, start: usize) -> Option<&'b str> {
        let (start_in, end_in) = self.find_match_loc_in_at(haystack, start)?;
        haystack.get(start_in..end_in)
    }

    pub fn find_match_out_at<'b>(&self, haystack: &'b str, start: usize) -> Option<&'b str> {
        let (start_out, end_out) = self.find_match_loc_out_at(haystack, start)?;
        haystack.get(start_out..end_out)
    }

    pub fn find_match_in<'b>(&self, haystack: &'b str) -> Option<&'b str> {
        self.find_match_in_at(haystack, 0)
    }

    pub fn find_match_out<'b>(&self, haystack: &'b str) -> Option<&'b str> {
        self.find_match_out_at(haystack, 0)
    }

    pub fn find_match_iter<'b>(&self, haystack: &'b str) -> Vec<&'b str> {
        let mut result = Vec::new();

        let mut start = 0;
        while let Some(mat) = self.find_match_loc_out_at(haystack, start) {
            let (start_out, end_out) = mat;
            result.push(haystack.get(start_out..end_out).unwrap());
            start = end_out;
        }

        result
    }
}

impl <'a> From<(&'a Regex, &'a Regex)> for RegexPair<'a> {
    fn from(pair: (&'a Regex, &'a Regex)) -> Self {
        Self::new(pair.0, pair.1)
    }
}

#[derive(Builder)]
#[builder(finish_fn(vis = "", name = build_internal))]
pub struct KVParser<'a> {
    #[builder(name = "with_scope_patterns", into)]
    pub scope: Option<RegexPair<'a>>,
    #[builder(name = "with_iter_patterns", into)]
    pub iter: RegexPair<'a>,
    #[builder(name = "with_key_pattern")]
    pub key: &'a Regex,
    #[builder(name = "with_value_pattern")]
    pub value: &'a Regex,
}

impl<'a> KVParser<'a> {
    pub fn parse(&self, haystack: &'a str) -> Option<Vec<(&'a str, &'a str)>> {
        let scope = self.scope.as_ref().map(|pair| pair.find_match_in(haystack)).unwrap_or_else(|| Some(haystack))?;

        let iters = self.iter.find_match_iter(scope).iter().map(|s| {
            let key = self.key.captures(s).unwrap().iter().skip(1).find(|x| x.is_some()).flatten().unwrap().as_str();
            let value = self.value.captures(s).unwrap().iter().skip(1).find(|x| x.is_some()).flatten().unwrap().as_str();
            (key, value)
        }).collect::<Vec<_>>();

        Some(iters)
    }
}

impl<'a, S: k_v_parser_builder::IsComplete> KVParserBuilder<'a, S> {
    pub fn build(self) -> KVParser<'a> {
        let instance = self.build_internal();

        assert_static_capture_groups!(instance.key, 1);
        assert_static_capture_groups!(instance.value, 1);

        instance
    }

    /// Shorthand for `self.build().parse(haystack)`.
    pub fn parse(self, haystack: &'a str) -> Option<Vec<(&'a str, &'a str)>> {
        self.build().parse(haystack)
    }
}

#[cfg(test)]
mod tests {
    use regex::regex;
    use super::*;

    #[test]
    fn regex_pair() {
        let str = "abcdef";
        let regex_pair = RegexPair::new(
            regex!("b"),
            regex!("d"),
        );

        let mat_loc_in = regex_pair.find_match_loc_in(str);
        assert_eq!(mat_loc_in, Some((2, 3)));

        let mat_loc_out = regex_pair.find_match_loc_out(str);
        assert_eq!(mat_loc_out, Some((1, 4)));

        let mat_in = regex_pair.find_match_in(str);
        assert_eq!(mat_in, Some("c"));

        let mat_out = regex_pair.find_match_out(str);
        assert_eq!(mat_out, Some("bcd"));
    }

    #[test]
    fn k_v_parser() {
        let data = r#"
            w,x [start] a,b || c,d || e,f [end] y,z
            "#;

        let parser = KVParser::builder()
            .with_scope_patterns((regex!(r"\[start\]"), regex!(r"\[end\]")))
            .with_iter_patterns((regex!(r"^|\|\s+"), regex!(r"\s+\||$")))
            .with_key_pattern(regex!(r"(\w),"))
            .with_value_pattern(regex!(r",(\w)"))
            .build();

        let result = parser.parse(data);

        assert_eq!(result, Some(vec![
            ("a", "b"),
            ("c", "d"),
            ("e", "f"),
        ]));
    }
}
