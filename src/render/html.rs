//! HTML renderer: a self-contained two-pane reference page.
//!
//! Left pane is an indented, expandable command tree (cli_doc-style: hover,
//! selected highlight, caret affordance). Right pane is a detail card with
//! description, usage, and an arguments/options table. Hash-routed per
//! command. All CSS and JS are inlined so the output is a single file.

use crate::schema::CliModel;

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
aside{padding:1rem;border-right:1px solid var(--border);overflow:auto}
aside h2{font-size:.75rem;text-transform:uppercase;letter-spacing:.05em;color:var(--muted);margin:0 0 .5rem}
#tree ul{list-style:none;margin:0;padding:0}
#tree .node>button{display:flex;width:100%;align-items:center;gap:.3rem;padding:.28rem .5rem;border:0;border-radius:6px;background:none;color:inherit;font:inherit;text-align:left;cursor:pointer}
#tree .node>button:hover{background:var(--hover)}
#tree .node>button[aria-current=true]{background:var(--accent);color:#fff;font-weight:600}
#tree .caret{width:.8em;flex:0 0 auto;transition:transform .15s;color:var(--muted)}
#tree .node.open>.caret{transform:rotate(90deg)}
#tree .node.leaf>.caret{visibility:hidden}
#tree .children{margin-left:.9em;border-left:1px solid var(--border);padding-left:.35em}
#tree .children[hidden]{display:none}
#tree .hidden{display:none}
article{padding:1.25rem 1.5rem;max-width:72rem}
.card{border:1px solid var(--border);border-radius:8px;padding:1rem;margin-bottom:1rem}
.card h2{margin-top:0}
.card h3{font-size:.8rem;text-transform:uppercase;letter-spacing:.05em;color:var(--muted);margin:.25rem 0 .5rem}
pre.usage{background:#164e63;color:#fff;padding:1rem;border-radius:8px;overflow-x:auto;margin:0}
table{border-collapse:collapse;width:100%}
th,td{border:1px solid var(--border);padding:.4rem .55rem;text-align:left;vertical-align:top}
th{background:var(--hover);font-size:.85rem}
code{overflow-wrap:anywhere}
.empty{color:var(--muted);font-style:italic}
@media (max-width:760px){main{display:block}aside{border-right:0;border-bottom:1px solid var(--border)}.search input{min-width:0;width:100%}}
"#;

const JS: &str = r##"
(()=>{"use strict";
const data=JSON.parse(document.getElementById("docli-data").textContent);
const treeEl=document.getElementById("tree");
const detailEl=document.getElementById("detail");
const searchEl=document.getElementById("search");
const all=[];
const slug=s=>s.toLowerCase().replace(/[^a-z0-9]+/g,"-").replace(/^-+|-+$/g,"");
function flatten(node,path){const full=path.concat(node.name);node.anchor=slug(full.join(" "));all.push(node);(node.subcommands||[]).forEach(c=>flatten(c,full));}
flatten(data,[]);
const el=(tag,text)=>{const e=document.createElement(tag);if(text!=null)e.textContent=text;return e;};
function show(node){
  location.hash=node.anchor;
  detailEl.replaceChildren();
  const head=el("section");head.className="card";
  head.append(el("h2",node.name));
  const desc=node.long_description||node.description;
  if(desc)head.append(el("p",desc));
  if(node.usage){const u=el("pre",node.usage);u.className="usage";head.append(el("h3","Usage"),u);}
  detailEl.append(head);
  const opts=node.options||[],args=node.arguments||[];
  if(opts.length||args.length){
    const sec=el("section");sec.className="card";sec.append(el("h3","Arguments & options"));
    const t=el("table"),hr=el("tr");
    ["Name","Short","Value","Required","Default","Choices","Description"].forEach(x=>hr.append(el("th",x)));
    const thead=el("thead");thead.append(hr);t.append(thead);
    const tb=el("tbody");
    args.forEach(a=>{const r=el("tr");r.id=a.name;
      r.append(el("td",a.name),el("td",""),el("td",""),el("td",a.required?"yes":"no"),el("td",a.default_value||""),el("td",(a.choices||[]).join(", ")),el("td",a.help||""));
      tb.append(r);});
    opts.forEach(o=>{const r=el("tr");r.id=o.name;
      r.append(el("td",o.long||""),el("td",o.short||""),el("td",o.value_name||""),el("td",o.required?"yes":"no"),el("td",o.default_value||""),el("td",(o.choices||[]).join(", ")),el("td",o.long_help||o.help||""));
      tb.append(r);});
    t.append(tb);sec.append(t);detailEl.append(sec);
  }
  if(node.epilogue){const n=el("section");n.className="card";n.append(el("h3","Notes"),el("p",node.epilogue));detailEl.append(n);}
}
function buildTree(node){
  const li=el("li");li.className="node"+(node.subcommands&&node.subcommands.length?"":" leaf");
  const caret=el("span","\u25b6");caret.className="caret";
  const btn=el("button");btn.type="button";btn.dataset.anchor=node.anchor;btn.append(caret,node.name);
  btn.addEventListener("click",()=>show(node));
  li.append(btn);
  if(node.subcommands&&node.subcommands.length){const ul=el("ul");ul.className="children";node.subcommands.forEach(c=>ul.append(buildTree(c)));li.append(ul);}
  return li;
}
const root=el("ul");root.append(buildTree(data));treeEl.append(root);
treeEl.addEventListener("click",e=>{
  const btn=e.target.closest("button");if(!btn)return;
  const li=btn.parentElement;const kids=li.querySelector(":scope>.children");if(!kids)return;
  li.classList.toggle("open");kids.hidden=!kids.hidden;
});
function select(node){document.querySelectorAll("#tree button").forEach(b=>b.setAttribute("aria-current",String(b.dataset.anchor===node.anchor)));show(node);}
function byAnchor(a){return all.find(x=>x.anchor===a)||data;}
searchEl.addEventListener("input",()=>{const q=searchEl.value.toLowerCase();
  document.querySelectorAll("#tree .node").forEach(li=>{const name=li.querySelector("button").textContent.toLowerCase();li.classList.toggle("hidden",!!q&&!name.includes(q));});
});
window.addEventListener("hashchange",()=>{const n=byAnchor(location.hash.slice(1));if(n)select(n);});
select(byAnchor(location.hash.slice(1)));
})();
"##;

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn render(model: &CliModel) -> String {
    let title = escape_html(&model.name);
    let version = escape_html(model.version.as_deref().unwrap_or(""));
    // Break any accidental `</script>` sequence inside the embedded JSON.
    let data = serde_json::to_string(model)
        .expect("serialize model")
        .replace("</", "<\\/");
    format!(
        "<!doctype html>\n\
<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{title} CLI Reference</title>\n<style>{css}</style>\n</head>\n<body>\n\
<header><a class=\"crumb\" href=\"../\">Documentation</a><h1>{title} CLI Reference</h1>{version_html}<label class=\"search\">Search <input id=\"search\" type=\"search\" autocomplete=\"off\" placeholder=\"commands and options\"></label></header>\n\
<main><aside><h2>Commands</h2><nav id=\"tree\"></nav></aside><article id=\"detail\" aria-live=\"polite\"></article></main>\n\
<script id=\"docli-data\" type=\"application/json\">{data}</script>\n<script>{js}</script>\n</body>\n</html>\n",
        css = CSS,
        js = JS,
        version_html = if version.is_empty() {
            String::new()
        } else {
            format!("<span class=\"meta\">Version {version}</span>")
        },
    )
}
