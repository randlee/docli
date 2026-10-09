
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

  function optionCard(item, title, shortName) {
    const card = el("article");
    card.className = "option-card";
    if (item.name) card.id = item.name;
    card.append(el("h3", title));
    const meta = el("p");
    meta.className = "option-card-meta";
    const bits = [];
    if (shortName) bits.push(shortName);
    if (item.value_name) bits.push(item.value_name);
    bits.push(item.required ? "required" : "optional");
    if (item.default_value) bits.push("default " + item.default_value);
    if (item.choices && item.choices.length) bits.push(item.choices.join(", "));
    meta.textContent = bits.join(" · ");
    card.append(meta);
    const help = item.long_help || item.help || "";
    if (help) card.append(el("p", help));
    return card;
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
      args.forEach((arg) => sec.append(optionCard(arg, arg.name, "")));
      opts.forEach((option) => {
        sec.append(optionCard(option, option.long || option.name || "", option.short || ""));
      });
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
