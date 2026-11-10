import { t, setLang, slotName, kindName } from "./i18n.js";
import { createPreview } from "./preview.js";
import { mockInvoke } from "./mock.js";

const tauri = window.__TAURI__;
const isTauri = Boolean(tauri?.core?.invoke);
const invoke = isTauri
  ? (cmd, args) => tauri.core.invoke(cmd, args)
  : (cmd, args) => mockInvoke(cmd, args);

const SECTIONS = ["screen", "border", "cursor", "window", "animation"];
const CATALOGUE_URL =
  "https://raw.githubusercontent.com/noctalia-dev/community-umbriel-shaders/catalogue/catalogue.json";
const CATALOGUE_RAW =
  "https://raw.githubusercontent.com/noctalia-dev/community-umbriel-shaders/";

const S = {
  view: "library",
  settings: null,
  draft: null,
  lib: null,
  status: null,
  catalogue: null,
  catalogueBusy: false,
  catalogueProgress: null,
  selectedId: null,
  query: "",
  section: "all",
  pendingSlot: null,
  familyPick: {},
  preview: null,
  previewCtx: null,
  busy: false,
};

const $ = (sel) => document.querySelector(sel);
const esc = (value) =>
  String(value ?? "").replace(/[&<>"']/g, (c) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  }[c]));

const fmtBytes = (n) => {
  if (!n) return "0 B";
  if (n < 1024) return `${n} B`;
  if (n < 1048576) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1048576).toFixed(1)} MB`;
};

function toast(message, detail, bad = false) {
  const host = $("#toasts");
  const el = document.createElement("div");
  el.className = `toast${bad ? " bad" : ""}`;
  el.innerHTML = `<div>${esc(message)}</div>${detail ? `<small>${esc(detail)}</small>` : ""}`;
  host.appendChild(el);
  setTimeout(() => el.remove(), bad ? 8000 : 4200);
}

function askModal({ title, body, confirm, danger = false, input = null, placeholder = "" }) {
  return new Promise((resolve) => {
    const modal = $("#modal");
    const shell = document.querySelector(".app");
    modal.hidden = false;
    shell.inert = true;
    modal.innerHTML = `
      <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="dlg-title">
        <h2 id="dlg-title">${esc(title)}</h2>
        ${body ? `<p>${body}</p>` : ""}
        ${input ? `<input id="dlg-input" type="text" placeholder="${esc(placeholder)}" style="width:100%;height:38px;padding:0 12px;background:var(--panel2);border:1px solid var(--line);font-family:var(--mono);font-size:10.5px;color:var(--ink);margin-top:8px">` : ""}
        <div class="actions">
          <button class="btn ghost" id="dlg-cancel">${esc(t("actions.cancel"))}</button>
          <button class="btn ${danger ? "danger" : "primary"}" id="dlg-ok">${esc(confirm)}</button>
        </div>
      </div>`;
    let onKey;
    const close = (value) => {
      document.removeEventListener("keydown", onKey);
      modal.hidden = true;
      modal.innerHTML = "";
      shell.inert = false;
      resolve(value);
    };
    $("#dlg-cancel").onclick = () => close(null);
    $("#dlg-ok").onclick = () => close(input ? $("#dlg-input").value.trim() : true);
    modal.onclick = (event) => {
      if (event.target === modal) close(null);
    };
    const field = $("#dlg-input");
    if (field) {
      field.focus();
      field.onkeydown = (event) => {
        if (event.key === "Enter") close(field.value.trim());
      };
    }
    onKey = (event) => {
      if (event.key === "Escape") close(null);
    };
    document.addEventListener("keydown", onKey);
  });
}

function compatibleSlots(entry) {
  return (S.lib?.slots || []).filter((slot) => slot.kind === entry.kind);
}

function slotFor(entry) {
  const slots = compatibleSlots(entry);
  if (S.pendingSlot) {
    const pending = slots.find((slot) => slot.key === S.pendingSlot);
    if (pending) return pending;
  }
  const used = slots.find((slot) => entry.usedBy.includes(slot.key));
  return used || slots[0] || null;
}

function selectedEntry() {
  return (S.lib?.entries || []).find((entry) => entry.id === S.selectedId) || null;
}

function filteredEntries() {
  const query = S.query.trim().toLowerCase();
  const rank = (kind) => {
    const index = SECTIONS.indexOf(kind);
    return index < 0 ? SECTIONS.length : index;
  };
  return (S.lib?.entries || [])
    .filter((entry) => {
      if (S.section !== "all" && entry.kind !== S.section) return false;
      if (!query) return true;
      const haystack = [entry.name, entry.kind, entry.author, entry.description, ...entry.usedBy.map(slotName)]
        .join(" ")
        .toLowerCase();
      return haystack.includes(query);
    })
    .sort((a, b) => rank(a.kind) - rank(b.kind) || a.name.localeCompare(b.name));
}

function sectionGroups() {
  const counts = new Map();
  for (const entry of S.lib?.entries || []) {
    counts.set(entry.kind, (counts.get(entry.kind) || 0) + 1);
  }
  return SECTIONS.filter((kind) => counts.has(kind)).map((kind) => ({
    kind,
    count: counts.get(kind),
  }));
}

function chooseSelection() {
  const list = filteredEntries();
  if (S.selectedId && list.some((entry) => entry.id === S.selectedId)) return;
  S.selectedId = list[0]?.id || null;
}

async function refresh() {
  S.lib = await invoke("library_scan");
  S.status = await invoke("umbriel_status");
  S.catalogue = await invoke("catalogue_info");
  chooseSelection();
}

/* ---------- chrome ---------- */

function renderChrome() {
  document.querySelectorAll("#nav button").forEach((button) => {
    const view = button.dataset.view;
    button.textContent = t(`nav.${view}`);
    const on = S.view === view;
    button.classList.toggle("on", on);
    button.setAttribute("aria-current", on ? "page" : "false");
  });
  document.querySelectorAll(".lang button").forEach((button) => {
    button.classList.toggle("on", button.dataset.lang === S.settings.language);
  });
  const status = $("#status");
  const on = S.status?.connected;
  status.classList.toggle("off", !on);
  status.querySelector("span").textContent = on ? t("status.on") : t("status.off");
  status.title = S.status?.error || "";
  $("#entry-count").textContent = t("count.shaders", { n: S.lib?.entries.length || 0 });
}

function renderTape() {
  const libraryEntries = (S.lib?.entries || []).filter((entry) => entry.source === "library");
  const canUndo = Boolean(S.lib?.canUndo);
  $("#tape").innerHTML = `
    <span>${esc(t("tape.origin"))}</span>
    <span>${esc(t("tape.library"))} <b>${libraryEntries.length}</b></span>
    <span>${esc(t("tape.size", { size: fmtBytes(S.lib?.libraryBytes || 0) }))}</span>
    <span class="ok">${esc(t("tape.generated"))}</span>
    <span class="spacer"></span>
    <button id="tape-undo" ${canUndo ? "" : "disabled"}>${esc(t("tape.undo"))}</button>
    <button id="tape-restore" class="warn">${esc(t("tape.restore"))}</button>`;
  $("#tape-undo").onclick = undo;
  $("#tape-restore").onclick = restore;
}

/* ---------- library ---------- */

function renderLibrary() {
  const view = $("#view");
  const entries = S.lib?.entries || [];
  if (!entries.length) {
    view.innerHTML = `
      <div class="empty">
        <div class="box">
          <h2>${esc(t("library.empty.title"))}</h2>
          <p>${esc(t("library.empty.body"))}</p>
          <code>${esc(S.lib?.libraryPath || S.settings.libraryPath)}</code>
          <div class="actions">
            <button class="btn primary" id="empty-catalog" ${S.catalogueBusy ? "disabled" : ""}>${esc(S.catalogueBusy ? t("catalogue.busy") : t("library.empty.catalog"))}</button>
            <button class="btn ghost" id="empty-install">${esc(t("library.empty.install"))}</button>
            <button class="btn ghost" id="empty-import">${esc(t("library.empty.import"))}</button>
          </div>
        </div>
      </div>`;
    $("#empty-catalog").onclick = downloadCatalogue;
    $("#empty-install").onclick = installStarters;
    $("#empty-import").onclick = importShader;
    return;
  }

  view.innerHTML = `
    <div class="library">
      <aside class="sections" id="sections"></aside>
      <div class="tablewrap">
        <div class="toolbar">
          <label class="search">
            <svg viewBox="0 0 14 14"><circle cx="6" cy="6" r="4.2"/><path d="M9.4 9.4L12.5 12.5"/></svg>
            <input id="q" type="search" placeholder="${esc(t("library.search"))}" value="${esc(S.query)}" aria-label="${esc(t("library.search"))}">
          </label>
          <span class="hint">${esc(t("detail.applyHint"))}</span>
        </div>
        <div class="tablehead">
          <span>${esc(t("table.no"))}</span>
          <span>${esc(t("table.shader"))}</span>
          <span>${esc(t("table.type"))}</span>
          <span>${esc(t("table.slot"))}</span>
          <span style="text-align:right">${esc(t("table.duration"))}</span>
        </div>
        <div class="rows" id="rows" role="listbox" aria-label="${esc(t("nav.library"))}"></div>
      </div>
      <aside class="detail" id="detail"></aside>
      <footer class="ruler" id="ruler"></footer>
    </div>`;

  $("#q").oninput = (event) => {
    S.query = event.target.value;
    chooseSelection();
    renderRows();
    renderDetail();
  };
  renderSections();
  renderRows();
  renderDetail();
  renderRuler();
}

function renderSections() {
  const host = $("#sections");
  if (!host) return;
  const groups = sectionGroups();
  const total = S.lib?.entries.length || 0;
  const installed = (S.lib?.entries || []).filter((entry) => entry.official).length;
  const button = S.catalogueBusy
    ? t("catalogue.busy")
    : installed
      ? t("catalogue.update")
      : t("catalogue.download");
  const status = S.catalogueBusy
    ? S.catalogueProgress?.total
      ? t("catalogue.progress", S.catalogueProgress)
      : t("catalogue.busy")
    : S.catalogue
      ? t("catalogue.installed", { n: installed, total: S.catalogue.total })
      : t("catalogue.local");
  const item = (kind, label, count) => `
    <button class="sec ${S.section === kind ? "on" : ""}" data-section="${kind}" aria-current="${S.section === kind ? "page" : "false"}">
      <span>${esc(label)}</span><b>${count}</b>
    </button>`;
  host.innerHTML = `
    <div class="sechead">${esc(t("sections.title"))}</div>
    <div class="seclist">
      ${item("all", t("sections.all"), total)}
      ${groups.map((group) => item(group.kind, t(`section.${group.kind}`), group.count)).join("")}
    </div>
    <div class="catbox">
      <div class="sechead">${esc(t("catalogue.title"))}</div>
      <p class="note" id="cat-status">${esc(status)}</p>
      <button class="btn ghost catbtn" id="cat-download" ${S.catalogueBusy ? "disabled" : ""}>${esc(button)}</button>
    </div>`;
  host.querySelectorAll("[data-section]").forEach((element) => {
    element.onclick = () => {
      S.section = element.dataset.section;
      chooseSelection();
      renderSections();
      renderRows();
      renderDetail();
    };
  });
  $("#cat-download").onclick = downloadCatalogue;
}

function renderRows() {
  const rows = $("#rows");
  if (!rows) return;
  const list = filteredEntries();
  if (!list.length) {
    rows.innerHTML = `<div class="empty" style="min-height:220px"><div class="box"><p>${esc(S.query ? t("library.noresults") : t("families.empty"))}</p></div></div>`;
    return;
  }
  const grouped = S.section === "all";
  const groups = grouped
    ? SECTIONS.map((kind) => ({
        kind,
        items: list.filter((entry) => entry.kind === kind),
      })).filter((group) => group.items.length)
    : [{ kind: S.section, items: list }];
  let index = 0;
  rows.innerHTML = groups
    .map((group) => {
      const head = grouped
        ? `<div class="group"><span>${esc(t(`section.${group.kind}`))}</span><b>${group.items.length}</b></div>`
        : "";
      return (
        head +
        group.items
          .map((entry) => {
            index += 1;
            const slot = entry.usedBy.length
              ? (S.lib.slots || []).find((item) => item.key === entry.usedBy[0])
              : null;
            const used = entry.usedBy.length > 0;
            return `
        <button class="row ${entry.id === S.selectedId ? "sel" : ""}" data-id="${esc(entry.id)}" role="option" aria-selected="${entry.id === S.selectedId}">
          <span class="idx">${String(index).padStart(2, "0")}</span>
          <span class="nm">${used ? '<i class="dot"></i>' : ""}${esc(entry.name)}</span>
          <span class="ty">${esc(kindName(entry.kind))}</span>
          <span class="sl">${used ? esc(slotName(entry.usedBy[0])) : esc(t("table.none"))}</span>
          <span class="du">${slot && slot.durationMs ? esc(t("detail.ms", { n: slot.durationMs })) : esc(t("table.none"))}</span>
        </button>`;
          })
          .join("")
      );
    })
    .join("");
  rows.onclick = (event) => {
    const row = event.target.closest("[data-id]");
    if (!row) return;
    S.selectedId = row.dataset.id;
    S.pendingSlot = null;
    renderRows();
    renderDetail();
  };
}

function renderDetail() {
  const host = $("#detail");
  if (!host) return;
  const entry = selectedEntry();
  if (!entry) {
    host.innerHTML = "";
    return;
  }
  if (S.preview) {
    S.preview.dispose();
    S.preview = null;
  }
  const slot = slotFor(entry);
  const slots = compatibleSlots(entry);
  const duration = slot?.durationMs || 280;
  const curve = slot?.curve || "easeoutcubic";
  const isAnimation = entry.kind === "animation";
  const curves = ["linear", "ease", "easein", "easeout", "easeinout", "easeincubic", "easeoutcubic", "easeinoutcubic", "snappy", "overshoot", "smooth"];

  host.innerHTML = `
    <div class="preview">
      <div class="prevlabel">${esc(t("detail.preview"))} <b>${esc(t("detail.live"))}</b></div>
      <div class="desk">
        <div class="win">
          <div class="win-bar"><i></i><i></i><i></i><span>${esc(entry.name)}</span></div>
          <div class="win-screen">
            <canvas id="pv"></canvas>
            <div class="preview-error" id="pverr" hidden></div>
          </div>
        </div>
      </div>
      <div class="prevfoot">
        <span>${esc(slot ? slotName(slot.key) : t("table.none"))}</span>
        <span>${isAnimation ? `${esc(t("detail.ms", { n: duration }))} · ${esc(curve)}` : esc(kindName(entry.kind))}</span>
      </div>
    </div>
    <div class="info">
      <h1>${esc(entry.name)}</h1>
      <div class="sub"><span class="src">${esc(t(entry.official ? "source.official" : entry.source === "library" ? "source.library" : "source.umbriel"))}</span>${entry.author ? ` · ${esc(entry.author)}` : ""} · ${esc(kindName(entry.kind))} · ${esc(entry.entryFn)}()</div>
      ${entry.description ? `<p class="entrydesc">${esc(entry.description)}</p>` : ""}
      <div class="kv">
        <div><small>${esc(t("detail.path"))}</small><strong class="path" title="${esc(entry.path)}">${esc(entry.path)}</strong></div>
        <div><small>${esc(t("table.slot"))}</small><strong>${entry.usedBy.length ? esc(entry.usedBy.map(slotName).join(", ")) : esc(t("table.none"))}</strong></div>
      </div>
      <div class="form">
        <div class="field wide">
          <div class="fieldrow">
            <label for="f-slot">${esc(t("detail.slot"))}</label>
            <button class="link danger" id="f-clear" data-slot="${esc(slot?.key || "")}" ${slot?.effect ? "" : "hidden"}>${esc(t("detail.clear"))}</button>
          </div>
          <select id="f-slot">
            ${slots.map((item) => `<option value="${esc(item.key)}" ${slot && item.key === slot.key ? "selected" : ""}>${esc(slotName(item.key))}</option>`).join("")}
          </select>
        </div>
        ${isAnimation ? `
        <div class="field">
          <label for="f-duration">${esc(t("detail.duration"))}</label>
          <input id="f-duration" type="number" min="1" max="10000" step="10" value="${duration}">
        </div>
        <div class="field">
          <label for="f-curve">${esc(t("detail.curve"))}</label>
          <select id="f-curve">${curves.map((name) => `<option value="${name}" ${name === curve ? "selected" : ""}>${name}</option>`).join("")}</select>
        </div>` : ""}
      </div>
      <div class="actions">
        <button class="btn primary" id="f-apply">${esc(t("detail.apply"))}</button>
        <button class="btn ghost" id="f-test">${esc(t("detail.test"))}</button>
        ${entry.source === "library" ? `<button class="btn danger" id="f-remove">${esc(t("detail.remove"))}</button>` : ""}
      </div>
    </div>`;

  const errorBox = $("#pverr");
  S.preview = createPreview($("#pv"), {
    onError: (message) => {
      if (!errorBox) return;
      errorBox.hidden = !message;
      if (message) {
        errorBox.innerHTML = `<div><b>${esc(t("detail.previewUnavailable"))}</b><br>${esc(String(message).slice(0, 420))}</div>`;
      }
    },
  });
  S.preview.setReducedMotion(S.settings.reduceMotion);
  loadPreview(entry, duration);

  $("#f-slot").onchange = (event) => {
    const next = (S.lib.slots || []).find((item) => item.key === event.target.value);
    if (!next) return;
    if (isAnimation) {
      $("#f-duration").value = next.durationMs || 280;
      $("#f-curve").value = next.curve || "easeoutcubic";
    }
    const clear = $("#f-clear");
    if (clear) {
      clear.dataset.slot = next.key;
      clear.hidden = !next.effect;
    }
  };
  if (isAnimation) {
    $("#f-duration").onchange = (event) => {
      const ms = Math.max(1, Math.min(10000, Number(event.target.value) || 280));
      event.target.value = ms;
      if (S.previewCtx) S.preview.setShader(S.previewCtx.source, S.previewCtx.entryFn, S.previewCtx.kind, ms);
    };
  }
  $("#f-apply").onclick = () => applySelected(entry);
  $("#f-test").onclick = () => S.preview?.restart();
  const clearBtn = $("#f-clear");
  if (clearBtn) clearBtn.onclick = () => clearSlot(clearBtn.dataset.slot);
  if (entry.source === "library") $("#f-remove").onclick = () => removeEntry(entry);
  renderRuler();
}

async function loadPreview(entry, duration) {
  try {
    const data = await invoke("shader_source", { id: entry.id });
    S.previewCtx = { source: data.source, entryFn: data.entryFn, kind: data.kind };
    S.preview?.setShader(data.source, data.entryFn, data.kind, duration);
  } catch (error) {
    const box = $("#pverr");
    if (box) {
      box.hidden = false;
      box.innerHTML = `<div><b>${esc(t("detail.previewUnavailable"))}</b><br>${esc(String(error).slice(0, 420))}</div>`;
    }
  }
}

function renderRuler() {
  const host = $("#ruler");
  if (!host) return;
  const active = (S.lib?.slots || []).filter((slot) => slot.kind === "animation" && slot.effect);
  const activeKey = slotFor(selectedEntry())?.key;
  if (!active.length) {
    host.innerHTML = `<div class="rhead"><span><b>${esc(t("ruler.title"))}</b></span></div><div class="note">${esc(t("ruler.empty"))}</div>`;
    return;
  }
  const total = active.reduce((sum, slot) => sum + (slot.durationMs || 0), 0) || 1;
  let cursor = 0;
  const segments = active
    .map((slot) => {
      const width = ((slot.durationMs || 0) / total) * 100;
      const left = cursor;
      cursor += width;
      return `<div class="seg ${slot.key === activeKey ? "on" : ""}" style="left:${left}%;width:${width}%" title="${esc(slotName(slot.key))}">
        <span class="n">${esc(slotName(slot.key))}</span><span class="v">${slot.durationMs || 0}</span>
      </div>`;
    })
    .join("");
  const ticks = [0, 25, 50, 75, 100]
    .map((pct, index) => `<span style="left:${pct}%">${Math.round((total * pct) / 100)}</span>`)
    .join("");
  host.innerHTML = `
    <div class="rhead">
      <span><b>${esc(t("ruler.title"))}</b> · ${esc(t("ruler.slots", { n: active.length }))}</span>
      <span>${esc(t("ruler.total", { n: total }))}</span>
    </div>
    <div class="track">${segments}</div>
    <div class="ticks">${ticks}</div>`;
}

/* ---------- slots ---------- */

function renderSlots() {
  const view = $("#view");
  const slots = S.lib?.slots || [];
  view.innerHTML = `
    <div class="page">
      <div class="pagehead">
        <div>
          <h1>${esc(t("slots.title"))}</h1>
          <p>${esc(t("slots.desc"))}</p>
        </div>
        <div class="side">
          <button class="btn ghost" id="slots-restore">${esc(t("actions.restore"))}</button>
        </div>
      </div>
      <div class="slotgrid">
        ${slots.map((slot) => `
          <div class="slotcard${slot.kind === "animation" && !slot.enabled ? " off" : ""}">
            <div class="top">
              <h3>${esc(slotName(slot.key))}</h3>
              <span class="kind">${esc(kindName(slot.kind))}</span>
            </div>
            <div class="cur ${slot.effect ? "" : "none"}">${esc(slot.effect || t("slots.none"))}</div>
            <div class="foot">
              <span>${slot.kind === "animation" && slot.effect ? `${slot.durationMs || 0} ms · ${esc(slot.curve || "easeoutcubic")}` : esc(slot.enabled ? t("slots.enabled") : t("slots.disabled"))}</span>
              <span class="acts">
                <button data-slot="${esc(slot.key)}">${esc(t("slots.change"))}</button>
                ${slot.effect ? `<button class="clear" data-clear="${esc(slot.key)}">${esc(t(slot.kind === "animation" ? "slots.clearAnim" : "slots.clearEffect"))}</button>` : ""}
              </span>
            </div>
          </div>`).join("")}
      </div>
    </div>`;
  $("#slots-restore").onclick = restore;
  view.querySelectorAll("[data-clear]").forEach((button) => {
    button.onclick = () => clearSlot(button.dataset.clear);
  });
  view.querySelectorAll("[data-slot]").forEach((button) => {
    button.onclick = () => {
      const slot = slots.find((item) => item.key === button.dataset.slot);
      if (!slot) return;
      S.pendingSlot = slot.key;
      S.section = slot.kind;
      S.query = "";
      S.view = "library";
      const candidate = (S.lib.entries || []).find((entry) => entry.kind === slot.kind);
      S.selectedId = candidate?.id || null;
      renderAll();
    };
  });
}

/* ---------- families ---------- */

function renderFamilies() {
  const view = $("#view");
  const entries = S.lib?.entries || [];
  const kinds = ["animation", "border", "window", "cursor", "screen"];
  view.innerHTML = `
    <div class="page">
      <div class="pagehead">
        <div>
          <h1>${esc(t("families.title"))}</h1>
          <p>${esc(t("families.desc"))}</p>
        </div>
      </div>
      <div class="famgrid">
        ${kinds.map((kind) => {
          const members = entries.filter((entry) => entry.kind === kind);
          const pick = S.familyPick[kind] || members[0]?.id;
          const targets = (S.lib?.slots || []).filter((slot) => slot.kind === kind);
          const countLabel = members.length === 1 ? t("families.memberOne") : t("families.members", { n: members.length });
          const applyLabel = targets.length === 1 ? t("families.applyOne") : t("families.apply", { n: targets.length });
          return `
            <div class="famcard">
              <div class="top"><h3>${esc(kindName(kind))}</h3><span class="count">${esc(countLabel)}</span></div>
              <p>${members.length ? esc(members.map((entry) => entry.name).slice(0, 4).join(" · ")) : esc(t("families.empty"))}</p>
              <div class="members">
                ${members.map((entry) => `<button class="member ${entry.id === pick ? "on" : ""}" data-kind="${esc(kind)}" data-id="${esc(entry.id)}">${esc(entry.name)}</button>`).join("")}
              </div>
              <div class="actions">
                <button class="btn primary" data-apply="${esc(kind)}" ${members.length && targets.length ? "" : "disabled"}>${esc(applyLabel)}</button>
              </div>
            </div>`;
        }).join("")}
        ${kinds.length % 2 === 1 ? '<div class="famfill"></div>' : ""}
      </div>
    </div>`;
  view.querySelectorAll(".member").forEach((button) => {
    button.onclick = () => {
      S.familyPick[button.dataset.kind] = button.dataset.id;
      renderFamilies();
    };
  });
  view.querySelectorAll("[data-apply]").forEach((button) => {
    button.onclick = () => applyFamily(button.dataset.apply);
  });
}

/* ---------- settings ---------- */

function renderSettings() {
  const view = $("#view");
  const draft = S.draft;
  const libraryEntries = (S.lib?.entries || []).filter((entry) => entry.source === "library");
  const dirty = draft.libraryPath !== S.settings.libraryPath || draft.originalConfig !== S.settings.originalConfig;
  view.innerHTML = `
    <div class="page settings">
      <div class="pagehead">
        <div>
          <h1>${esc(t("settings.title"))}</h1>
          <p>${esc(t("settings.desc"))}</p>
        </div>
        <div class="side">
          <span id="dirty" class="note bad" ${dirty ? "" : "hidden"}>${esc(t("settings.unsaved"))}</span>
          <button class="btn primary" id="save-settings" ${dirty ? "" : "disabled"}>${esc(t("settings.save"))}</button>
        </div>
      </div>

      <section class="section">
        <div>
          <h2>${esc(t("settings.library.title"))}</h2>
          <p class="desc">${esc(t("settings.library.desc"))}</p>
        </div>
        <div class="body">
          <label class="field">
            <span class="note">${esc(t("settings.library.path"))}</span>
            <input type="text" id="set-library" value="${esc(draft.libraryPath)}">
          </label>
          <div class="row2">
            <button class="btn ghost" id="open-library">${esc(t("settings.library.open"))}</button>
            <button class="btn ghost" id="rescan-library">${esc(t("settings.library.rescan"))}</button>
            <button class="btn ghost" id="install-examples">${esc(t("settings.library.install"))}</button>
            <button class="btn ghost" id="import-shader">${esc(t("settings.library.import"))}</button>
          </div>
          <div class="stats">
            <div><small>${esc(t("settings.library.count"))}</small><b>${libraryEntries.length}</b></div>
            <div><small>${esc(t("settings.library.size"))}</small><b>${esc(fmtBytes(S.lib?.libraryBytes || 0))}</b></div>
          </div>
          <div class="note">${esc(t("settings.library.generated"))}<br><b style="color:var(--ink)">${esc(S.lib?.generated || "")}</b></div>
        </div>
      </section>

      <section class="section">
        <div>
          <h2>${esc(t("settings.umbriel.title"))}</h2>
          <p class="desc">${esc(t("settings.umbriel.desc"))}</p>
        </div>
        <div class="body">
          <label class="field">
            <span class="note">${esc(t("settings.umbriel.config"))}</span>
            <input type="text" id="set-config" value="${esc(draft.originalConfig)}">
          </label>
          <div class="row2">
            <button class="btn ghost" id="validate-config">${esc(t("settings.umbriel.validate"))}</button>
          </div>
          <div class="note" id="validate-note">${S.status?.connected ? esc(t("status.compiled", { n: S.status.compiled || 0 })) : esc(S.status?.error || t("status.off"))}</div>
        </div>
      </section>

      <section class="section">
        <div>
          <h2>${esc(t("settings.interface.title"))}</h2>
          <p class="desc">${esc(t("settings.interface.desc"))}</p>
        </div>
        <div class="body">
          <span class="note">${esc(t("settings.interface.language"))}</span>
          <div class="segmented" id="set-lang">
            <button data-lang="es" class="${draft.language === "es" ? "on" : ""}">Español</button>
            <button data-lang="en" class="${draft.language === "en" ? "on" : ""}">English</button>
          </div>
          <label class="check">
            <input type="checkbox" id="set-motion" ${draft.reduceMotion ? "checked" : ""}>
            <span>${esc(t("settings.interface.reduce"))}<small>${esc(t("settings.interface.reduceHelp"))}</small></span>
          </label>
        </div>
      </section>

      <section class="section">
        <div><h2>${esc(t("settings.about.title"))}</h2></div>
        <div class="body about">
          <span>${esc(t("settings.about.version"))} <b>0.1.0</b></span>
          <span>${esc(t("settings.about.data"))} <b>${esc(S.settings.libraryPath)}</b></span>
          <span>${esc(t("settings.about.license"))} <b>MIT</b></span>
        </div>
      </section>
    </div>`;

  $("#set-library").oninput = (event) => {
    S.draft.libraryPath = event.target.value;
    $("#dirty").hidden = false;
    $("#save-settings").disabled = false;
  };
  $("#set-config").oninput = (event) => {
    S.draft.originalConfig = event.target.value;
    $("#dirty").hidden = false;
    $("#save-settings").disabled = false;
  };
  $("#save-settings").onclick = saveSettings;
  $("#open-library").onclick = () => invoke("library_reveal").catch((error) => toast(t("toast.error"), String(error), true));
  $("#rescan-library").onclick = async () => {
    await refresh();
    renderAll();
  };
  $("#install-examples").onclick = installStarters;
  $("#import-shader").onclick = importShader;
  $("#validate-config").onclick = async () => {
    const note = $("#validate-note");
    try {
      const result = await invoke("validate_config");
      note.className = "note ok";
      note.textContent = result.message || t("settings.umbriel.valid");
    } catch (error) {
      note.className = "note bad";
      note.textContent = `${t("settings.umbriel.invalid")}: ${String(error).slice(0, 300)}`;
    }
  };
  $("#set-lang").onclick = async (event) => {
    const button = event.target.closest("[data-lang]");
    if (!button) return;
    S.draft.language = button.dataset.lang;
    S.settings.language = button.dataset.lang;
    setLang(button.dataset.lang);
    try {
      S.settings = await invoke("settings_save", { settings: S.settings });
    } catch (error) {
      toast(t("toast.error"), String(error), true);
    }
    renderAll();
  };
  $("#set-motion").onchange = async (event) => {
    S.draft.reduceMotion = event.target.checked;
    S.settings.reduceMotion = event.target.checked;
    try {
      S.settings = await invoke("settings_save", { settings: S.settings });
    } catch (error) {
      toast(t("toast.error"), String(error), true);
    }
    renderAll();
  };
}

/* ---------- actions ---------- */

function updateCatalogueStatus() {
  const status = $("#cat-status");
  if (status && S.catalogueProgress?.total) {
    status.textContent = t("catalogue.progress", S.catalogueProgress);
  }
}

async function downloadCatalogue() {
  if (!isTauri) {
    toast(t("catalogue.desktop"));
    return;
  }
  if (S.catalogueBusy) return;
  S.catalogueBusy = true;
  S.catalogueProgress = { done: 0, total: 0 };
  renderAll();
  let failed = 0;
  try {
    const response = await fetch(CATALOGUE_URL, { cache: "no-store" });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const text = await response.text();
    const catalogue = JSON.parse(text);
    const revision = catalogue?.source?.revision;
    if (!/^[0-9a-f]{40}$/.test(revision || "")) {
      throw new Error("La revisión del catálogo no es válida.");
    }
    S.catalogue = await invoke("catalogue_save", { json: text });
    const effects = (catalogue.effects || []).filter((effect) => effect?.id);
    S.catalogueProgress = { done: 0, total: effects.length };
    updateCatalogueStatus();
    const queue = effects.slice();
    const worker = async () => {
      while (queue.length) {
        const effect = queue.shift();
        const [kind, slug] = effect.id.split("/");
        try {
          const path = `${effect.id}/shader.glsl`;
          const record = catalogue.files?.[path];
          if (!record?.sha256) throw new Error("sin hash");
          const file = await fetch(`${CATALOGUE_RAW}${revision}/${path}`, {
            cache: "no-store",
          });
          if (!file.ok) throw new Error(`HTTP ${file.status}`);
          const source = await file.text();
          await invoke("library_install_shader", {
            kind,
            slug,
            source,
            sha256: record.sha256,
          });
        } catch {
          failed += 1;
        }
        S.catalogueProgress.done += 1;
        updateCatalogueStatus();
      }
    };
    await Promise.all(Array.from({ length: 6 }, worker));
    toast(
      t("catalogue.done", { n: effects.length - failed }),
      failed ? t("catalogue.partial", { n: failed }) : String(revision).slice(0, 12),
      failed > 0
    );
  } catch (error) {
    toast(t("catalogue.error"), String(error), true);
  } finally {
    S.catalogueBusy = false;
    S.catalogueProgress = null;
    await refresh();
    renderAll();
  }
}

async function saveSettings() {
  try {
    S.settings = await invoke("settings_save", { settings: S.draft });
    S.draft = { ...S.settings };
    await refresh();
    toast(t("toast.saved"));
    renderAll();
  } catch (error) {
    toast(t("toast.error"), String(error), true);
  }
}

async function applySelected(entry) {
  const slotKey = $("#f-slot")?.value;
  if (!slotKey) return;
  const duration = Number($("#f-duration")?.value) || 280;
  const curve = $("#f-curve")?.value || "easeoutcubic";
  try {
    const result = await invoke("apply_shader", {
      req: { slot: slotKey, entryId: entry.id, durationMs: duration, curve },
    });
    toast(`${t("toast.applied")} · ${slotName(slotKey)}`, result.message);
    S.pendingSlot = null;
    await refresh();
    renderAll();
  } catch (error) {
    toast(t("toast.error"), String(error), true);
  }
}

async function applyFamily(kind) {
  const members = (S.lib?.entries || []).filter((entry) => entry.kind === kind);
  const pick = S.familyPick[kind] || members[0]?.id;
  if (!pick) return;
  try {
    const result = await invoke("apply_family", { entryId: pick, kind });
    toast(`${t("toast.applied")} · ${kindName(kind)}`, result.message);
    await refresh();
    renderAll();
  } catch (error) {
    toast(t("toast.error"), String(error), true);
  }
}

async function clearSlot(key) {
  if (!key) return;
  try {
    const result = await invoke("clear_slot", { slot: key });
    toast(t("toast.cleared"), result.message);
    S.pendingSlot = null;
    await refresh();
    renderAll();
  } catch (error) {
    toast(t("toast.error"), String(error), true);
  }
}

async function undo() {
  try {
    const result = await invoke("undo_change");
    toast(t("toast.undone"), result.message);
    await refresh();
    renderAll();
  } catch (error) {
    toast(t("toast.error"), String(error), true);
  }
}

async function restore() {
  const ok = await askModal({
    title: t("dialog.restore.title"),
    body: esc(t("dialog.restore.body")),
    confirm: t("dialog.restore.confirm"),
    danger: true,
  });
  if (!ok) return;
  try {
    const result = await invoke("restore_original");
    toast(t("toast.restored"), result.message);
    await refresh();
    renderAll();
  } catch (error) {
    toast(t("toast.error"), String(error), true);
  }
}

async function installStarters() {
  try {
    const count = await invoke("library_install_starters");
    toast(t("toast.seeded", { n: count }));
    await refresh();
    renderAll();
  } catch (error) {
    toast(t("toast.error"), String(error), true);
  }
}

async function importShader() {
  const path = await askModal({
    title: t("settings.library.import"),
    body: esc(t("toast.importHint")),
    confirm: t("settings.library.import"),
    input: true,
    placeholder: "~/shaders/aurora.glsl",
  });
  if (!path) return;
  try {
    const count = await invoke("library_import", { path });
    toast(t("toast.imported", { n: count }));
    await refresh();
    renderAll();
  } catch (error) {
    toast(t("toast.error"), String(error), true);
  }
}

async function removeEntry(entry) {
  const ok = await askModal({
    title: t("detail.remove"),
    body: esc(t("toast.confirmRemove", { name: entry.name })),
    confirm: t("detail.remove"),
    danger: true,
  });
  if (!ok) return;
  try {
    await invoke("library_remove", { id: entry.id });
    toast(t("toast.removed"), entry.name);
    S.selectedId = null;
    await refresh();
    renderAll();
  } catch (error) {
    toast(t("toast.error"), String(error), true);
  }
}

/* ---------- render ---------- */

function renderAll() {
  renderChrome();
  renderTape();
  if (S.view === "library") renderLibrary();
  else if (S.view === "slots") renderSlots();
  else if (S.view === "families") renderFamilies();
  else renderSettings();
}

function wireChrome() {
  $("#nav").onclick = (event) => {
    const button = event.target.closest("[data-view]");
    if (!button) return;
    S.view = button.dataset.view;
    S.pendingSlot = null;
    renderAll();
    $("#view").focus();
  };
  document.querySelector(".lang").onclick = async (event) => {
    const button = event.target.closest("[data-lang]");
    if (!button || button.dataset.lang === S.settings.language) return;
    S.settings.language = button.dataset.lang;
    S.draft.language = button.dataset.lang;
    setLang(button.dataset.lang);
    try {
      S.settings = await invoke("settings_save", { settings: S.settings });
    } catch (error) {
      toast(t("toast.error"), String(error), true);
    }
    renderAll();
  };
  document.addEventListener("keydown", (event) => {
    const mod = event.ctrlKey || event.metaKey;
    if (mod && event.key.toLowerCase() === "k") {
      event.preventDefault();
      $("#q")?.focus();
    }
    if (mod && event.key === "Enter" && S.view === "library") {
      const entry = selectedEntry();
      if (entry) {
        event.preventDefault();
        applySelected(entry);
      }
    }
    if (S.view === "library" && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
      if (["INPUT", "SELECT", "TEXTAREA"].includes(document.activeElement?.tagName)) return;
      const list = filteredEntries();
      const index = list.findIndex((entry) => entry.id === S.selectedId);
      if (index === -1) return;
      const next = list[index + (event.key === "ArrowDown" ? 1 : -1)];
      if (next) {
        event.preventDefault();
        S.selectedId = next.id;
        renderRows();
        renderDetail();
      }
    }
  });

  if (tauri?.window?.getCurrentWindow) {
    const win = tauri.window.getCurrentWindow();
    $("#win-min").onclick = () => win.minimize();
    $("#win-max").onclick = () => win.toggleMaximize();
    $("#win-close").onclick = () => win.close();
  } else {
    ["#win-min", "#win-max", "#win-close"].forEach((sel) => {
      $(sel).onclick = () => toast("Ventana del navegador", "Los controles funcionan en la app de escritorio.", true);
    });
  }
}

async function boot() {
  try {
    S.settings = await invoke("settings_get");
    S.draft = { ...S.settings };
    setLang(S.settings.language);
    const initial = new URLSearchParams(location.search).get("view");
    if (["library", "slots", "families", "settings"].includes(initial)) S.view = initial;
    await refresh();
    wireChrome();
    renderAll();
  } catch (error) {
    $("#view").innerHTML = `<div class="empty"><div class="box"><h2>${esc(t("toast.error"))}</h2><p>${esc(String(error))}</p></div></div>`;
  }
}

boot();
