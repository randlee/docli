
(() => {
  "use strict";
  const treeEl = document.getElementById("tree");
  const detailEl = document.getElementById("detail");
  const searchEl = document.getElementById("search");
  const layoutEl = document.getElementById("quarto-content");
  const sidebarToggle = document.getElementById("sidebar-toggle");

  if (sidebarToggle && layoutEl) {
    sidebarToggle.addEventListener("click", () => {
      const collapsed = layoutEl.classList.toggle("sidebar-collapsed");
      sidebarToggle.setAttribute("aria-expanded", collapsed ? "false" : "true");
    });
  }

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
    head.className = "card quarto-title-block";
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
    li.className = hasKids ? "node open sidebar-item" : "node leaf sidebar-item";
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
    cmd.className = "docli-cmd sidebar-link";
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
  root.className = "sidebar-menu-list";
  root.append(buildTree(data));
  treeEl.append(root);
  searchEl.addEventListener("input", applySearch);
  window.addEventListener("hashchange", routeFromHash);
  routeFromHash();
})();
