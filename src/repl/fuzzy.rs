// fuzzy matching for variables and templates

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use crate::data::VARIABLES;

/// search variables with fuzzy matching
pub fn search_variables(pattern: &str) -> Vec<&'static str> {
    if pattern.is_empty() {
        // return all variables sorted alphabetically
        let mut vars: Vec<&str> = VARIABLES.iter().copied().collect();
        vars.sort();
        return vars;
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pat = Pattern::parse(pattern, CaseMatching::Ignore, Normalization::Smart);

    let mut matches: Vec<(u32, &str)> = VARIABLES
        .iter()
        .filter_map(|var| {
            let mut buf = Vec::new();
            let haystack = Utf32Str::new(var, &mut buf);
            pat.score(haystack, &mut matcher).map(|score| (score, *var))
        })
        .collect();

    // sort by score (descending), then alphabetically
    matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));

    matches.into_iter().map(|(_, var)| var).collect()
}

/// search templates with fuzzy matching
pub fn search_templates(pattern: &str, templates: &[String]) -> Vec<String> {
    if pattern.is_empty() {
        return templates.to_vec();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pat = Pattern::parse(pattern, CaseMatching::Ignore, Normalization::Smart);

    let mut matches: Vec<(u32, &String)> = templates
        .iter()
        .filter_map(|t| {
            let mut buf = Vec::new();
            let haystack = Utf32Str::new(t, &mut buf);
            pat.score(haystack, &mut matcher).map(|score| (score, t))
        })
        .collect();

    matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));

    matches.into_iter().map(|(_, t)| t.clone()).collect()
}
