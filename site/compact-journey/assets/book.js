/*
This file is part of Compact.
Copyright (C) 2026 Midnight Foundation
SPDX-License-Identifier: Apache-2.0
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

(() => {
  "use strict";

  const root = document.documentElement;
  const prefix = document.body.dataset.bookRoot;
  const chapterId = document.body.dataset.chapterId;
  const themeButton = document.getElementById("theme-toggle");
  const readKey = "compact-journey-read-v1";
  const readStored = (key, fallback) => {
    try { return JSON.parse(localStorage.getItem(key)) ?? fallback; }
    catch { return fallback; }
  };
  const save = (key, value) => {
    try { localStorage.setItem(key, JSON.stringify(value)); }
    catch { /* Reading remains available when browser storage is disabled. */ }
  };
  const updateTheme = (theme) => {
    root.dataset.theme = theme;
    themeButton.textContent = theme === "dark" ? "Light" : "Dark";
    themeButton.setAttribute("aria-label", "Switch to " + (theme === "dark" ? "light" : "dark") + " theme");
  };
  const storedTheme = readStored("compact-journey-theme", "light");
  updateTheme(storedTheme === "dark" ? "dark" : "light");
  themeButton.addEventListener("click", () => {
    const theme = root.dataset.theme === "dark" ? "light" : "dark";
    updateTheme(theme);
    save("compact-journey-theme", theme);
  });

  const menu = document.getElementById("menu-toggle");
  const sidebar = document.getElementById("book-sidebar");
  const mobile = window.matchMedia?.("(max-width: 740px)");
  const setMenu = (open, restoreFocus = false) => {
    document.body.classList.toggle("menu-open", open);
    menu.setAttribute("aria-expanded", String(open));
    sidebar.inert = Boolean(mobile?.matches && !open);
    if (open) document.getElementById("chapter-search").focus();
    else if (restoreFocus) menu.focus();
  };
  setMenu(false);
  menu.addEventListener("click", () => {
    const open = !document.body.classList.contains("menu-open");
    setMenu(open, !open);
  });
  mobile?.addEventListener("change", () => setMenu(false));
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && document.body.classList.contains("menu-open")) {
      setMenu(false, true);
    }
  });

  const readValue = readStored(readKey, []);
  const read = new Set(Array.isArray(readValue) ? readValue.filter((v) => typeof v === "string") : []);
  const chapterLinks = [...document.querySelectorAll("[data-chapter]")];
  const updateProgress = () => {
    chapterLinks.forEach((link) => link.classList.toggle("is-read", read.has(link.dataset.chapter)));
    const known = new Set(chapterLinks.map((link) => link.dataset.chapter));
    const count = [...read].filter((id) => known.has(id)).length;
    document.getElementById("reading-progress").textContent = count + " of " + known.size + " chapters read";
  };
  updateProgress();
  if (chapterId) save("compact-journey-last", chapterId);
  const readButton = document.getElementById("mark-read");
  if (readButton) {
    const syncReadButton = () => {
      readButton.setAttribute("aria-pressed", String(read.has(chapterId)));
      readButton.textContent = read.has(chapterId) ? "Chapter read ✓" : "Mark chapter as read";
    };
    syncReadButton();
    readButton.addEventListener("click", () => {
      if (read.has(chapterId)) read.delete(chapterId);
      else read.add(chapterId);
      save(readKey, [...read]);
      syncReadButton();
      updateProgress();
    });
  }
  const resume = document.getElementById("continue-reading");
  const last = readStored("compact-journey-last", null);
  if (resume && chapterLinks.some((link) => link.dataset.chapter === last)) {
    resume.href = "chapters/" + last + ".html";
    resume.hidden = false;
  }
  const printButton = document.getElementById("print-book");
  if (printButton) printButton.addEventListener("click", () => window.print());

  const chapterSearch = document.getElementById("chapter-search");
  let searchIndex = null;
  let searchPromise = null;
  chapterSearch.addEventListener("input", async () => {
    const query = chapterSearch.value.trim().toLowerCase();
    if (query && !searchIndex) {
      if (!searchPromise) searchPromise = fetch(prefix + "search.json").then((response) => {
        if (!response.ok) throw Error("Search index unavailable");
        return response.json();
      }).then((data) => { searchIndex = data; }).catch(() => { searchIndex = []; });
      await searchPromise;
    }
    const current = chapterSearch.value.trim().toLowerCase();
    let count = 0;
    chapterLinks.forEach((link) => {
      const record = searchIndex?.find((item) => item.slug === link.dataset.chapter);
      const text = record ? record.title + " " + record.phase + " " + record.text : link.textContent;
      const match = !current || text.toLowerCase().includes(current);
      link.closest("li").hidden = !match;
      if (match) count++;
    });
    document.querySelectorAll(".chapter-nav h2").forEach((heading) => {
      heading.hidden = [...heading.nextElementSibling.children].every((li) => li.hidden);
    });
    document.getElementById("chapter-search-status").textContent = count + " chapters match" +
      (query && searchIndex?.length === 0 ? ". Full-text index unavailable, searching titles." : "");
  });

  const stages = [
    {title: "Source admission", owner: "Shared Compact frontend", artifact: "Typed source with disclosure and capability decisions.", example: "An unsupported language pragma or undeclared witness disclosure fails here.", next: "Accepted source still needs a valid target representation."},
    {title: "Typed lowering", owner: "Versioned Rust IR and checked lowering", artifact: "An admitted program with explicit types, operations and source locations.", example: "K1 and P-256 operands retain distinct types before syntax emission.", next: "Compile the generated Rust through its ordinary public entry points."},
    {title: "Native execution", owner: "Generated contract and ledger runtime", artifact: "Result, state, gas, witness effects and private state.", example: "Compare full state bytes and cost fields against an independent TypeScript capture.", next: "Native execution leaves transcript construction to the next check."},
    {title: "Recording and replay", owner: "Checked effects and runtime recorder", artifact: "Ordered ledger operations and the replayed public transition.", example: "A Unit-valued mutation must survive even when its value is unused.", next: "Bind the recording to the circuit statement and installed operation."},
    {title: "SDK preflight", owner: "Pinned ZKIR interpreter", artifact: "An exact serialized preimage checked against raw IR.", example: "Substitute a ledger write, event order or witness value and check the expected refusal.", next: "Generate and independently verify the cryptographic proof."},
    {title: "Proof and verification", owner: "Prover and independent verifier", artifact: "Fresh proof with verifier identity tied to exact IR and pinned parameters.", example: "Refuse a different verifier, a changed binding and corrupted proof bytes.", next: "Test the transaction consumer with its own statement and fee material."},
    {title: "Funded application", owner: "Ledger 9 transaction API", artifact: "A separately proved, fee-funded transaction accepted under default strictness.", example: "Inspect the installed operation, applied state and ordered ledger events.", next: "Reusing the verified transaction must exercise replay protection."},
    {title: "Replay refusal", owner: "Ledger state transition rules", artifact: "IntentAlreadyExists with unchanged serialized state in the selected gates.", example: "Check that replay emits no duplicate events and leaves ledger bytes intact.", next: "Record the source revision, platform, gate and coverage limit."}
  ];
  const pipeline = document.querySelector(".pipeline");
  if (pipeline) {
    const detail = document.getElementById("pipeline-detail");
    const selectStage = (index) => {
      [...pipeline.children].forEach((button, i) => button.setAttribute("aria-pressed", String(i === index)));
      detail.replaceChildren();
      const stage = stages[index];
      const title = document.createElement("h2");
      title.textContent = stage.title;
      const dl = document.createElement("dl");
      [["Owner", stage.owner], ["Produces", stage.artifact], ["Example", stage.example], ["Next check", stage.next]].forEach(([term, value]) => {
        const dt = document.createElement("dt");
        const dd = document.createElement("dd");
        dt.textContent = term;
        dd.textContent = value;
        dl.append(dt, dd);
      });
      detail.append(title, dl);
    };
    stages.forEach((stage, index) => {
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = stage.title;
      button.setAttribute("aria-pressed", "false");
      button.addEventListener("click", () => selectStage(index));
      pipeline.append(button);
    });
    selectStage(0);
  }

  const panel = document.getElementById("commit-panel");
  const panelTitle = document.getElementById("commit-panel-title");
  const panelContent = document.getElementById("commit-panel-content");
  let revisions;
  let openedCommit;
  let detailTrigger;
  function showCommit(record) {
    panelTitle.textContent = record.subject;
    const description = document.createElement("p");
    description.textContent = record.description || (
      record.url ? "Commit recorded in the edition’s published history." :
        "Local revision preserved at this edition’s cutoff."
    );
    const metadata = document.createElement("dl");
    for (const [label, value] of [
      ["Authored", record.authored.slice(0, 10)],
      ["Committed", record.committed.slice(0, 10)],
      ["Commit", record.sha]
    ]) {
      const term = document.createElement("dt");
      term.textContent = label;
      const detail = document.createElement("dd");
      detail.textContent = value;
      metadata.append(term, detail);
    }
    panelContent.replaceChildren(description);
    if (record.message) {
      const heading = document.createElement("h3");
      heading.textContent = "Commit message";
      const message = document.createElement("div");
      message.className = "commit-message";
      message.textContent = record.message;
      panelContent.append(heading, message);
    }
    const link = document.createElement("a");
    link.href = record.url || prefix + "revisions.html#commit-" + record.sha;
    link.textContent = record.url ? "View commit on GitHub ↗" : "View local revision";
    panelContent.append(metadata, link);
  }
  document.addEventListener("click", async (event) => {
    const trigger = event.target.closest("button[data-commit]");
    if (!trigger) return;
    detailTrigger = trigger;
    const sha = trigger.dataset.commit;
    openedCommit = sha;
    panelTitle.textContent = "Commit details";
    panelContent.textContent = "Loading commit details…";
    if (!panel.open) panel.showModal();
    panel.querySelector("button").focus();
    try {
      revisions ??= fetch(prefix + "sources/revisions.json").then((response) => {
        if (!response.ok) throw Error("Commit descriptions unavailable");
        return response.json();
      });
      const record = (await revisions)[sha];
      if (openedCommit !== sha) return;
      if (!record) throw Error("Commit description missing");
      showCommit(record);
    } catch {
      revisions = undefined;
      if (openedCommit === sha) {
        panelContent.textContent = "Details could not load. Close the panel and try again, or follow the commit title.";
      }
    }
  });
  panel.addEventListener("close", () => {
    openedCommit = undefined;
    detailTrigger?.focus();
  });

  const table = document.getElementById("commit-results");
  if (!table) return;
  const fields = Object.fromEntries(["query", "phase", "topic", "publication", "order"].map((key) =>
    [key, document.getElementById("commit-" + key)]));
  const status = document.getElementById("commit-status");
  const previous = document.getElementById("commit-prev");
  const next = document.getElementById("commit-next");
  const phaseNames = Object.fromEntries([...fields.phase.options].map((option) => [option.value, option.text]));
  let page = 0;
  let records = [];
  let filtered = [];
  const pageSize = 40;
  const cell = (value) => {
    const td = document.createElement("td");
    td.textContent = value;
    return td;
  };
  const render = () => {
    const pages = Math.max(1, Math.ceil(filtered.length / pageSize));
    page = Math.min(page, pages - 1);
    table.replaceChildren();
    filtered.slice(page * pageSize, (page + 1) * pageSize).forEach((record) => {
      const tr = document.createElement("tr");
      const description = cell("");
      const subject = document.createElement("a");
      subject.className = "commit-subject";
      subject.textContent = record.subject;
      subject.href = record.publication === "published"
        ? "https://github.com/betsudo/compact/commit/" + record.sha
        : prefix + "revisions.html#commit-" + record.sha;
      const details = document.createElement("button");
      details.type = "button";
      details.className = "commit-details";
      details.dataset.commit = record.sha;
      details.textContent = "Details";
      details.setAttribute("aria-haspopup", "dialog");
      details.setAttribute("aria-label", "Commit details for " + record.subject);
      const tags = document.createElement("div");
      tags.className = "tags";
      tags.textContent = record.topics.join(" · ");
      description.append(subject, details, tags);
      const scope = cell("");
      const label = document.createElement("span");
      label.className = "scope " + record.publication;
      label.textContent = record.publication === "published" ? "Published" : "Local at cutoff";
      scope.append(label);
      tr.append(cell(record.authored.slice(0, 10)), description, cell(phaseNames[record.phase] || record.phase), scope);
      table.append(tr);
    });
    status.textContent = filtered.length.toLocaleString() + " matching commits of " + records.length.toLocaleString();
    document.getElementById("commit-page").textContent = "Page " + (page + 1) + " of " + pages;
    previous.disabled = page === 0;
    next.disabled = page + 1 >= pages;
  };
  const filter = () => {
    const query = fields.query.value.trim().toLowerCase();
    filtered = records.filter((record) =>
      (!query || (record.sha + " " + record.subject + " " + record.topics.join(" ")).toLowerCase().includes(query)) &&
      (!fields.phase.value || record.phase === fields.phase.value) &&
      (!fields.topic.value || record.topics.includes(fields.topic.value)) &&
      (!fields.publication.value || record.publication === fields.publication.value));
    if (fields.order.value === "newest") filtered.reverse();
    page = 0;
    const params = new URLSearchParams();
    Object.entries(fields).forEach(([key, element]) => {
      if (element.value && !(key === "order" && element.value === "oldest")) params.set(key, element.value);
    });
    history.replaceState(null, "", location.pathname + (params.size ? "?" + params : ""));
    render();
  };
  const params = new URLSearchParams(location.search);
  Object.entries(fields).forEach(([key, element]) => {
    if (params.has(key)) element.value = params.get(key);
    element.addEventListener(element.tagName === "INPUT" ? "input" : "change", filter);
  });
  previous.addEventListener("click", () => { page--; render(); });
  next.addEventListener("click", () => { page++; render(); });
  document.getElementById("commit-filters").addEventListener("submit", (event) => event.preventDefault());
  document.getElementById("commit-filters").addEventListener("reset", () => setTimeout(filter, 0));
  fetch(prefix + "commits.json").then((response) => {
    if (!response.ok) throw Error("Catalogue request failed");
    return response.json();
  }).then((data) => { records = data; filter(); }).catch(() => {
    status.textContent = "The catalogue could not load. Use the JSON download below or reload the page.";
    previous.disabled = true;
    next.disabled = true;
  });
})();
