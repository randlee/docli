//! HTML renderer: a self-contained two-pane reference page.
//!
//! The left pane is an indented command tree. Each command is a `.node`
//! whose `.docli-caret` and `.docli-cmd` buttons are siblings: the caret
//! toggles only that node, and the command name selects it. The right pane
//! is the detail card. `#docli-data` feeds the tree and the detail panel.
//! `#docli-search` is the only input to search filtering. CSS and JS are
//! inlined.

use std::collections::BTreeSet;
use std::fmt::Write;

use crate::schema::CliModel;
use crate::search::{anchor_for_path, search_index, unique_anchor};

const CSS: &str = r#"
:root{color-scheme:light dark;--bg:#fff;--fg:#1a1a2e;--border:#ddd;--hover:#f0f0f0;--accent:#007acc;--muted:#666}
@media (prefers-color-scheme:dark){:root{--bg:#16181d;--fg:#e6e6e6;--border:#333;--hover:#23262e;--accent:#4da3ff;--muted:#999}}
*{box-sizing:border-box}
body{margin:0;font:16px/1.5 system-ui,-apple-system,Segoe UI,sans-serif;background:var(--bg);color:var(--fg)}
header{display:flex;flex-wrap:wrap;align-items:baseline;gap:.75rem;padding:.75rem 1.25rem;border-bottom:1px solid var(--border)}
header h1{font-size:1.2rem;margin:0}
.meta{color:var(--muted);font-size:.85rem}
.crumb{color:var(--accent);text-decoration:none;font-size:.9rem}
.search{margin-left:auto}
.search input{padding:.35rem .6rem;border:1px solid var(--border);border-radius:6px;font:inherit;background:var(--bg);color:var(--fg);min-width:16rem}
main{display:grid;grid-template-columns:minmax(16rem,28%) 1fr;min-height:calc(100vh - 3.2rem)}
.pane{min-width:0}
aside.pane{padding:1rem;border-right:1px solid var(--border);overflow:auto}
aside.pane h2{font-size:.75rem;text-transform:uppercase;letter-spacing:.05em;color:var(--muted);margin:0 0 .5rem}
article.pane{padding:1.25rem 1.5rem;max-width:72rem}
#tree ul{list-style:none;margin:0;padding:0}
#tree .docli-caret,#tree .docli-cmd{border:0;background:none;color:inherit;font:inherit;cursor:pointer}
#tree .docli-caret{width:1.1rem;height:1.4rem;padding:0;color:var(--muted);vertical-align:middle}
#tree .docli-caret::before{content:"\25B6";display:inline-block;font-size:.65rem;transition:transform .15s}
#tree .node.open>.docli-caret::before{transform:rotate(90deg)}
#tree .node.leaf>.docli-caret{visibility:hidden}
#tree .docli-cmd{display:inline-flex;align-items:center;padding:.28rem .5rem;border-radius:6px;text-align:left}
#tree .docli-cmd:hover{background:var(--hover)}
#tree .docli-cmd[aria-current=true]{background:var(--accent);color:#fff;font-weight:600}
#tree .children{margin-left:.9em;border-left:1px solid var(--border);padding-left:.35em}
#tree .children[hidden]{display:none}
#tree .hidden{display:none}
.card{border:1px solid var(--border);border-radius:8px;padding:1rem;margin-bottom:1rem}
.card h2{margin-top:0}
.card h3{font-size:.8rem;text-transform:uppercase;letter-spacing:.05em;color:var(--muted);margin:.25rem 0 .5rem}
pre.usage{background:#164e63;color:#fff;padding:1rem;border-radius:8px;overflow-x:auto;margin:0}
table{border-collapse:collapse;width:100%}
th,td{border:1px solid var(--border);padding:.4rem .55rem;text-align:left;vertical-align:top}
th{background:var(--hover);font-size:.85rem}
code{overflow-wrap:anywhere}
@media (max-width:760px){main{display:block}aside.pane{border-right:0;border-bottom:1px solid var(--border)}.search input{min-width:0;width:100%}}
"#;

// Slug logic here must match `search::anchor_for_path`: lowercase, collapse
// non-alphanumeric runs to `-`, and trim `-`. Search reads `#docli-search`
// only; `#docli-data` builds the tree and the detail panel.
const JS: &str = r##"
(() => {
  "use strict";
  const treeEl = document.getElementById("tree");
  const detailEl = document.getElementById("detail");
  const searchEl = document.getElementById("search");

  function report(message) {
    if (!detailEl) return;
    detailEl.replaceChildren();
    const note = document.createElement("p");
    note.id = "docli-status";
    note.textContent = message;
    detailEl.append(note);
  }

  let data;
  let searchIndex;
  try {
    const dataNode = document.getElementById("docli-data");
    const searchNode = document.getElementById("docli-search");
    if (!dataNode || !searchNode) {
      throw new Error("embedded JSON is missing");
    }
    data = JSON.parse(dataNode.textContent);
    searchIndex = JSON.parse(searchNode.textContent);
    if (data === null || typeof data !== "object" || Array.isArray(data) || !Array.isArray(searchIndex)) {
      throw new Error("embedded JSON is invalid");
    }
  } catch (err) {
    const why = err && err.message ? err.message : "invalid JSON";
    report("Could not read this page's command data. " + why);
    return;
  }

  const all = [];
  const usedAnchors = new Set();

  // Mirror `search::slug`: only ASCII alnum are kept; other code points are separators.
  function slug(value) {
    let out = "";
    let pendingDash = false;
    for (const ch of value) {
      const code = ch.charCodeAt(0);
      const isAsciiAlnum =
        (code >= 48 && code <= 57) ||
        (code >= 65 && code <= 90) ||
        (code >= 97 && code <= 122);
      if (isAsciiAlnum) {
        if (pendingDash) {
          out += "-";
          pendingDash = false;
        }
        out += ch.toLowerCase();
      } else if (out.length) {
        pendingDash = true;
      }
    }
    return out;
  }

  // Same preorder rule as `search::unique_anchor`: the first slug wins, and
  // each later collision takes the next free -2, -3, ... anchor.
  function uniqueAnchor(base) {
    if (!usedAnchors.has(base)) {
      usedAnchors.add(base);
      return base;
    }
    let suffix = 2;
    let candidate = base + "-" + suffix;
    while (usedAnchors.has(candidate)) {
      suffix += 1;
      candidate = base + "-" + suffix;
    }
    usedAnchors.add(candidate);
    return candidate;
  }

  function flatten(node, path) {
    const full = path.concat(node.name);
    node.anchor = uniqueAnchor(slug(full.join(" ")));
    all.push(node);
    (node.subcommands || []).forEach((child) => flatten(child, full));
  }
  flatten(data, []);

  function el(tag, text) {
    const node = document.createElement(tag);
    if (text != null) node.textContent = text;
    return node;
  }

  function show(node, notice) {
    if (!notice && location.hash.slice(1) !== node.anchor) {
      location.hash = node.anchor;
    }
    detailEl.replaceChildren();
    if (notice) {
      const note = el("p", notice);
      note.id = "docli-status";
      note.className = "meta";
      detailEl.append(note);
    }
    const head = el("section");
    head.className = "card";
    head.append(el("h2", node.name));
    const desc = node.long_description || node.description;
    if (desc) head.append(el("p", desc));
    if (node.usage) {
      const usage = el("pre", node.usage);
      usage.className = "usage";
      head.append(el("h3", "Usage"), usage);
    }
    detailEl.append(head);
    const opts = node.options || [];
    const args = node.arguments || [];
    if (opts.length || args.length) {
      const sec = el("section");
      sec.className = "card";
      sec.append(el("h3", "Arguments & options"));
      const table = el("table");
      const header = el("tr");
      ["Name", "Short", "Value", "Required", "Default", "Choices", "Description"].forEach((label) => {
        header.append(el("th", label));
      });
      const thead = el("thead");
      thead.append(header);
      table.append(thead);
      const body = el("tbody");
      args.forEach((arg) => {
        const row = el("tr");
        row.id = arg.name;
        row.append(
          el("td", arg.name),
          el("td", ""),
          el("td", ""),
          el("td", arg.required ? "yes" : "no"),
          el("td", arg.default_value || ""),
          el("td", (arg.choices || []).join(", ")),
          el("td", arg.help || "")
        );
        body.append(row);
      });
      opts.forEach((option) => {
        const row = el("tr");
        row.id = option.name;
        row.append(
          el("td", option.long || option.name || ""),
          el("td", option.short || ""),
          el("td", option.value_name || ""),
          el("td", option.required ? "yes" : "no"),
          el("td", option.default_value || ""),
          el("td", (option.choices || []).join(", ")),
          el("td", option.long_help || option.help || "")
        );
        body.append(row);
      });
      table.append(body);
      sec.append(table);
      detailEl.append(sec);
    }
    if (node.epilogue) {
      const notes = el("section");
      notes.className = "card";
      notes.append(el("h3", "Notes"), el("p", node.epilogue));
      detailEl.append(notes);
    }
  }

  function buildTree(node) {
    const hasKids = Array.isArray(node.subcommands) && node.subcommands.length > 0;
    const li = el("li");
    li.className = hasKids ? "node open" : "node leaf";
    li.dataset.anchor = node.anchor;
    const caret = el("button");
    caret.type = "button";
    caret.className = "docli-caret";
    caret.setAttribute("aria-expanded", hasKids ? "true" : "false");
    caret.addEventListener("click", () => {
      const kids = li.querySelector(":scope > .children");
      if (!kids) return;
      const open = li.classList.toggle("open");
      caret.setAttribute("aria-expanded", open ? "true" : "false");
      kids.hidden = !open;
    });
    const cmd = el("button", node.name);
    cmd.type = "button";
    cmd.className = "docli-cmd";
    cmd.dataset.anchor = node.anchor;
    cmd.addEventListener("click", () => select(node));
    li.append(caret, cmd);
    if (hasKids) {
      const ul = el("ul");
      ul.className = "children";
      node.subcommands.forEach((child) => ul.append(buildTree(child)));
      li.append(ul);
    }
    return li;
  }

  function select(node, notice) {
    document.querySelectorAll("#tree .docli-cmd").forEach((button) => {
      button.setAttribute("aria-current", String(button.dataset.anchor === node.anchor));
    });
    show(node, notice);
  }

  function routeFromHash() {
    const anchor = location.hash.slice(1);
    if (anchor === "") {
      select(data);
      return;
    }
    const node = all.find((item) => item.anchor === anchor);
    if (!node) {
      select(data, "No command matches this link.");
      return;
    }
    select(node);
  }

  function matchingAnchors(query) {
    const q = query.toLowerCase();
    if (q === "") return null;
    const hits = new Set();
    for (const entry of searchIndex) {
      const matched = (entry.terms || []).some((term) => {
        const value = String(term).toLowerCase();
        if (value === "") return false;
        return value.includes(q) || q.includes(value);
      });
      if (matched) {
        hits.add(entry.anchor);
        (entry.ancestors || []).forEach((ancestor) => hits.add(ancestor));
      }
    }
    return hits;
  }

  function applySearch() {
    const hits = matchingAnchors(searchEl.value);
    document.querySelectorAll("#tree li.node").forEach((li) => {
      const visible = hits === null || hits.has(li.dataset.anchor);
      li.classList.toggle("hidden", !visible);
      if (hits === null || !visible) return;
      const kids = li.querySelector(":scope > .children");
      if (!kids) return;
      li.classList.add("open");
      const caret = li.querySelector(":scope > .docli-caret");
      if (caret) caret.setAttribute("aria-expanded", "true");
      kids.hidden = false;
    });
    let status = document.getElementById("docli-search-status");
    if (hits !== null && hits.size === 0) {
      if (!status) {
        status = el("p", "No matching commands.");
        status.id = "docli-search-status";
        status.className = "meta";
        treeEl.before(status);
      } else {
        status.textContent = "No matching commands.";
      }
    } else if (status) {
      status.remove();
    }
  }

  treeEl.replaceChildren();
  const root = el("ul");
  root.append(buildTree(data));
  treeEl.append(root);
  searchEl.addEventListener("input", applySearch);
  window.addEventListener("hashchange", routeFromHash);
  routeFromHash();
})();
"##;

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Serializes `value` for an inline JSON script tag.
///
/// This cannot fail for a valid [`CliModel`]: serde writes plain JSON values,
/// and the `</` escape is a [`String`] replacement.
fn embed_json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value)
        .expect("serialize embedded JSON")
        .replace("</", "<\\/")
}

fn render_tree(model: &CliModel) -> String {
    let mut names = Vec::new();
    let mut used = BTreeSet::new();
    let mut out = String::from("<ul>");
    render_node(model, &mut names, &mut used, &mut out);
    out.push_str("</ul>");
    out
}

/// Appends the HTML for `command` and its subcommands.
///
/// Writing into `out` cannot fail for a valid [`CliModel`]: `out` is a
/// [`String`], and [`std::fmt::Write`] for [`String`] does not return an error.
fn render_node(
    command: &CliModel,
    names: &mut Vec<String>,
    used: &mut BTreeSet<String>,
    out: &mut String,
) {
    names.push(command.name.clone());
    let anchor = unique_anchor(&anchor_for_path(names), used);
    let has_children = !command.subcommands.is_empty();
    let class = if has_children {
        "node open"
    } else {
        "node leaf"
    };
    let expanded = if has_children { "true" } else { "false" };
    let name = escape_html(&command.name);
    write!(
        out,
        "<li class=\"{class}\" data-anchor=\"{anchor}\"><button type=\"button\" class=\"docli-caret\" aria-expanded=\"{expanded}\"></button><button type=\"button\" class=\"docli-cmd\" data-anchor=\"{anchor}\">{name}</button>"
    )
    .expect("write html");
    if has_children {
        out.push_str("<ul class=\"children\">");
        for sub in &command.subcommands {
            render_node(sub, names, used, out);
        }
        out.push_str("</ul>");
    }
    out.push_str("</li>");
    names.pop();
}

/// Renders `model` as a self-contained two-pane HTML page.
pub fn render(model: &CliModel) -> String {
    let title = escape_html(&model.name);
    let version = escape_html(model.version.as_deref().unwrap_or(""));
    let version_html = if version.is_empty() {
        String::new()
    } else {
        format!("<span class=\"meta\">Version {version}</span>")
    };
    let data = embed_json(model);
    let search = embed_json(&search_index(model));
    let tree = render_tree(model);
    format!(
        "<!doctype html>\n\
<html lang=\"en\">\n\
<head>\n\
<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>{title} CLI Reference</title>\n\
<style>{css}</style>\n\
</head>\n\
<body>\n\
<header><a class=\"crumb\" href=\"../\">Documentation</a><h1>{title} CLI Reference</h1>{version_html}<label class=\"search\">Search <input id=\"search\" type=\"search\" autocomplete=\"off\" placeholder=\"commands and options\"></label></header>\n\
<main>\n\
<aside class=\"pane\"><h2>Commands</h2><nav id=\"tree\">{tree}</nav></aside>\n\
<article class=\"pane\" id=\"detail\" aria-live=\"polite\"></article>\n\
</main>\n\
<script id=\"docli-data\" type=\"application/json\">{data}</script>\n\
<script id=\"docli-search\" type=\"application/json\">{search}</script>\n\
<script>\n{js}\n</script>\n\
</body>\n\
</html>\n",
        css = CSS,
        js = JS,
    )
}

#[cfg(test)]
mod html_anchors {
    use super::render;
    use crate::schema::CliModel;

    #[test]
    fn static_tree_suffixes_colliding_anchors() {
        let model: CliModel = serde_json::from_str(
            r#"{"name":"demo","subcommands":[{"name":"Run Once"},{"name":"run-once"}]}"#,
        )
        .expect("parse model");
        let html = render(&model);
        assert!(html.contains("data-anchor=\"demo-run-once\""));
        assert!(html.contains("data-anchor=\"demo-run-once-2\""));
        assert!(html.contains("\"anchor\":\"demo-run-once-2\""));
    }
}
