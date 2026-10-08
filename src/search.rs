//! Search index embedded in the generated HTML page.
//!
//! [`search_index`] walks a [`CliModel`] into [`SearchEntry`] values.
//! [`matching_anchors`] reports which command anchors a query reveals,
//! including ancestors, without a browser. The HTML renderer embeds the
//! index as `#docli-search` and does not rebuild it from `#docli-data`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::schema::CliModel;

/// Searchable terms for one command and its ancestor anchors.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchEntry {
    /// Slash-free slug of this command's path.
    pub anchor: String,
    /// Ancestor anchors from the root down to the parent.
    pub ancestors: Vec<String>,
    /// Command, option, and argument strings used for matching.
    pub terms: Vec<String>,
}

/// Builds one [`SearchEntry`] per command in preorder.
///
/// Terms are the command name, each option's `name`, `long`, and `short`,
/// and each argument `name`. Empty strings are omitted so a blank term
/// cannot match every query.
///
/// # Examples
///
/// ```
/// let model: docli::CliModel = serde_json::from_str(
///     r#"{"name":"demo","subcommands":[{"name":"run"}]}"#,
/// )?;
/// let index = docli::search_index(&model);
/// assert_eq!(index[1].anchor, "demo-run");
/// assert_eq!(index[1].ancestors, vec!["demo".to_string()]);
/// Ok::<(), serde_json::Error>(())
/// ```
pub fn search_index(model: &CliModel) -> Vec<SearchEntry> {
    let mut index = Vec::new();
    let mut names = Vec::new();
    let mut ancestors = Vec::new();
    walk(model, &mut names, &mut ancestors, &mut index);
    index
}

/// Returns matching command anchors plus every ancestor anchor.
///
/// An empty `query` matches nothing. After both sides are lowercased, a term
/// matches when either string contains the other.
///
/// # Examples
///
/// ```
/// let model: docli::CliModel = serde_json::from_str(
///     r#"{"name":"demo","subcommands":[{"name":"run","options":[{"name":"verbose","long":"--verbose"}]}]}"#,
/// )?;
/// let index = docli::search_index(&model);
/// let hits = docli::matching_anchors(&index, "run");
/// assert!(hits.contains("demo"));
/// assert!(hits.contains("demo-run"));
/// assert!(docli::matching_anchors(&index, "").is_empty());
/// Ok::<(), serde_json::Error>(())
/// ```
pub fn matching_anchors(index: &[SearchEntry], query: &str) -> BTreeSet<String> {
    let mut hits = BTreeSet::new();
    if query.is_empty() {
        return hits;
    }
    let query = query.to_lowercase();
    for entry in index {
        let matched = entry.terms.iter().any(|term| {
            let term = term.to_lowercase();
            !term.is_empty() && (term.contains(&query) || query.contains(&term))
        });
        if matched {
            hits.insert(entry.anchor.clone());
            hits.extend(entry.ancestors.iter().cloned());
        }
    }
    hits
}

/// Slash-free slug for the command path `names`.
///
/// Joins `names` with one space, lowercases, collapses every
/// non-alphanumeric run to a single `-`, and trims `-` from both ends.
pub(crate) fn anchor_for_path(names: &[String]) -> String {
    slug(&names.join(" "))
}

fn slug(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut pending_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash {
                out.push('-');
                pending_dash = false;
            }
            out.push(ch.to_ascii_lowercase());
        } else if !out.is_empty() {
            pending_dash = true;
        }
    }
    out
}

fn walk(
    command: &CliModel,
    names: &mut Vec<String>,
    ancestors: &mut Vec<String>,
    index: &mut Vec<SearchEntry>,
) {
    names.push(command.name.clone());
    let anchor = anchor_for_path(names);
    index.push(SearchEntry {
        anchor: anchor.clone(),
        ancestors: ancestors.clone(),
        terms: terms_for(command),
    });
    ancestors.push(anchor);
    for sub in &command.subcommands {
        walk(sub, names, ancestors, index);
    }
    ancestors.pop();
    names.pop();
}

fn terms_for(command: &CliModel) -> Vec<String> {
    let mut terms = Vec::new();
    push_term(&mut terms, &command.name);
    for option in &command.options {
        push_term(&mut terms, &option.name);
        if let Some(long) = &option.long {
            push_term(&mut terms, long);
        }
        if let Some(short) = &option.short {
            push_term(&mut terms, short);
        }
    }
    for argument in &command.arguments {
        push_term(&mut terms, &argument.name);
    }
    terms
}

fn push_term(terms: &mut Vec<String>, value: &str) {
    if !value.is_empty() {
        terms.push(value.to_string());
    }
}

#[cfg(test)]
mod search_index {
    use super::{matching_anchors, search_index};
    use crate::schema::CliModel;

    fn model(json: &str) -> CliModel {
        serde_json::from_str(json).expect("parse model")
    }

    fn nested_verbose() -> CliModel {
        model(
            r#"{
                "name": "demo",
                "subcommands": [{
                    "name": "run",
                    "options": [{
                        "name": "verbose",
                        "long": "--verbose",
                        "short": "-v"
                    }]
                }]
            }"#,
        )
    }

    #[test]
    fn run_query_includes_demo_and_demo_run() {
        let index = search_index(&nested_verbose());
        let hits = matching_anchors(&index, "run");
        assert!(hits.contains("demo"));
        assert!(hits.contains("demo-run"));
    }

    #[test]
    fn verbose_query_includes_owner_and_ancestors() {
        let index = search_index(&nested_verbose());
        let run = index
            .iter()
            .find(|entry| entry.anchor == "demo-run")
            .expect("run entry");
        assert_eq!(run.ancestors, vec!["demo".to_string()]);
        assert!(run.terms.iter().any(|term| term == "--verbose"));
        assert!(run.terms.iter().any(|term| term == "-v"));
        assert!(run.terms.iter().any(|term| term == "verbose"));

        let hits = matching_anchors(&index, "--verbose");
        assert!(hits.contains("demo-run"));
        assert!(hits.contains("demo"));
    }

    #[test]
    fn empty_query_matches_nothing() {
        let index = search_index(&nested_verbose());
        assert!(matching_anchors(&index, "").is_empty());
    }

    #[test]
    fn unmatched_query_is_empty() {
        let index = search_index(&nested_verbose());
        assert!(matching_anchors(&index, "zzzz-no-such-term").is_empty());
    }

    #[test]
    fn parent_option_match_omits_unrelated_child() {
        let index = search_index(&model(
            r#"{
                "name": "demo",
                "options": [{"name": "file", "long": "--path"}],
                "subcommands": [{"name": "run"}]
            }"#,
        ));
        let hits = matching_anchors(&index, "--path");
        assert!(hits.contains("demo"));
        assert!(!hits.contains("demo-run"));
    }

    #[test]
    fn anchor_lowercases_and_collapses_symbols() {
        let index = search_index(&model(
            r#"{"name":"Demo","subcommands":[{"name":"Run Once"},{"name":"a--b"}]}"#,
        ));
        assert_eq!(index[0].anchor, "demo");
        assert!(index[0].ancestors.is_empty());
        assert_eq!(index[1].anchor, "demo-run-once");
        assert_eq!(index[1].ancestors, vec!["demo".to_string()]);
        assert_eq!(index[2].anchor, "demo-a-b");
    }

    #[test]
    fn argument_name_matches_its_owner() {
        let index = search_index(&model(
            r#"{"name":"demo","arguments":[{"name":"config"}],"subcommands":[{"name":"run"}]}"#,
        ));
        let hits = matching_anchors(&index, "config");
        assert!(hits.contains("demo"));
        assert!(!hits.contains("demo-run"));
    }
}
