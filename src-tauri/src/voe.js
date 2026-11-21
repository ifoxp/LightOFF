// Runs inside the VOE site window. Finds the schedule for the saved address and hands it to
// the app by navigating to https://lightoff.invalid/<kind>#<json>; the app cancels that
// navigation and reads the payload. The page gets no IPC access.
(() => {
  if (window.top !== window || window.__lightoff) return;
  window.__lightoff = true;

  const CFG = __LIGHTOFF_CFG__;
  const outbox = [];
  let sending = false;

  function send(kind, data) {
    outbox.push("https://lightoff.invalid/" + kind + "#" + encodeURIComponent(JSON.stringify(data)));
    if (!sending) flush();
  }
  function flush() {
    const next = outbox.shift();
    if (!next) {
      sending = false;
      return;
    }
    sending = true;
    try {
      window.location.href = next;
    } catch (e) {
      /* ignore */
    }
    setTimeout(flush, 200);
  }

  const norm = (s) =>
    String(s || "")
      .toLowerCase()
      .replace(/[«»"'’`]/g, "")
      .replace(/\s+/g, " ")
      .trim();

  // ---------- schedule table ----------

  function halfStates(cell) {
    const cls = cell.className || "";
    if (!cell.classList.contains("has_disconnection")) return ["on", "on"];
    const kind = /probab|maybe|possible|likely|yellow/i.test(cls) ? "maybe" : "off";
    if (cell.classList.contains("full_hour")) return [kind, kind];
    const blocks = cell.querySelectorAll(".hour_block");
    if (blocks.length >= 2) {
      return [0, 1].map((i) => {
        const b = blocks[i];
        const any = b.classList.contains("has_any") || b.classList.contains("has_disconnection");
        if (!any) return "on";
        return /probab|maybe|possible|likely|yellow/i.test(b.className) ? "maybe" : kind;
      });
    }
    if (/first_half|half_first|first-half/i.test(cls)) return [kind, "on"];
    if (/second_half|half_second|second-half/i.test(cls)) return ["on", kind];
    return [kind, kind];
  }

  function parse(root) {
    const days = [];
    const classes = new Set();
    root.querySelectorAll(".disconnection-detailed-table-container").forEach((table) => {
      let day = null;
      for (const c of table.children) {
        const cl = c.classList;
        if (cl.contains("head")) continue;
        if (cl.contains("day_col")) {
          day = { label: c.textContent.trim(), cells: [] };
          days.push(day);
          continue;
        }
        if (!day || !cl.contains("cell") || day.cells.length >= 24) continue;
        cl.forEach((x) => classes.add(x));
        c.querySelectorAll(".hour_block").forEach((b) => b.classList.forEach((x) => classes.add("hb:" + x)));
        day.cells.push(halfStates(c));
      }
    });
    const queueText = [...root.querySelectorAll(".disconnection-detailed-table > p, .disconnection-detailed-table p")]
      .map((p) => p.textContent.trim())
      .find((s) => /\d+\.\d+\s*черг/i.test(s));
    const queue = queueText ? (queueText.match(/\d+\.\d+/) || [null])[0] : null;
    return { days: days.filter((d) => d.cells.length === 24), queue, classes: [...classes].sort() };
  }

  let lastSent = "";
  function report(root, how) {
    const res = parse(root);
    if (!res.days.length) return false;
    const sig = JSON.stringify(res.days);
    if (sig === lastSent) return true;
    lastSent = sig;
    send("schedule", Object.assign(res, { how, url: location.href }));
    if (bar) bar.say("Графік зчитано, передаю в LightOFF…");
    return true;
  }

  // ---------- Drupal AJAX search (same requests the page makes itself) ----------

  function commands(text) {
    let t = String(text || "").trim();
    if (t.startsWith("<textarea")) {
      const ta = new DOMParser().parseFromString(t, "text/html").querySelector("textarea");
      t = ta ? ta.value : t;
    }
    return JSON.parse(t);
  }

  async function fetchText(url, init) {
    const r = await fetch(url, Object.assign({ credentials: "same-origin" }, init || {}));
    if (!r.ok) throw new Error("HTTP " + r.status + " " + url);
    return r.text();
  }

  function pick(items, wanted) {
    const w = norm(wanted);
    const parsed = items
      .map((it) => {
        const doc = new DOMParser().parseFromString(String(it.label || ""), "text/html");
        const el = doc.querySelector("[data-id]");
        return el ? { id: el.getAttribute("data-id"), value: String(it.value || el.textContent || "") } : null;
      })
      .filter(Boolean);
    if (!parsed.length) return null;
    const exact = parsed.find((p) => norm(p.value) === w);
    if (exact) return exact;
    const word = parsed.find((p) => new RegExp("(^|[\\s,.(])" + w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + "($|[\\s,.)])").test(norm(p.value)));
    return word || parsed[0];
  }

  function autocompleteBases(form) {
    const bases = [];
    const attr = form.querySelector('[name="city"]')?.getAttribute("data-autocomplete-path");
    if (attr) bases.push(attr.replace(/read_city.*$/, ""));
    const action = new URL(form.getAttribute("action") || location.pathname, location.href).pathname.replace(/\/$/, "");
    bases.push(action + "/autocomplete/");
    bases.push("/disconnection/detailed/autocomplete/");
    return [...new Set(bases)];
  }

  async function lookup(base, level, query, parent) {
    const url = base + "read_" + level + (parent ? "/" + parent : "") + "?q=" + encodeURIComponent(query);
    const items = commands(await fetchText(url, { headers: { Accept: "application/json, text/javascript, */*; q=0.01", "X-Requested-With": "XMLHttpRequest" } }));
    if (!Array.isArray(items)) throw new Error("автопідказка повернула не список");
    return pick(items, query);
  }

  async function autoSearch(form) {
    let base = null;
    let city = null;
    for (const b of autocompleteBases(form)) {
      try {
        city = await lookup(b, "city", CFG.city);
        base = b;
        break;
      } catch (e) {
        /* try the next path */
      }
    }
    if (!base) throw new Error("не знайдено автопідказку адрес на сайті");
    if (!city) throw new Error("місто «" + CFG.city + "» не знайдено");
    const street = await lookup(base, "street", CFG.street, city.id);
    if (!street) throw new Error("вулицю «" + CFG.street + "» не знайдено");
    const house = await lookup(base, "house", CFG.house, street.id);
    if (!house) throw new Error("будинок «" + CFG.house + "» не знайдено");
    send("log", { text: "адреса: " + city.value + ", " + street.value + ", " + house.value });

    const body = new URLSearchParams();
    form.querySelectorAll("input[name], select[name]").forEach((el) => {
      if ((el.type === "radio" || el.type === "checkbox") && !el.checked) return;
      if (el.type === "submit") return;
      body.set(el.name, el.value);
    });
    const searchBtn = form.querySelector('[name="search"]');
    const extra = {
      search_type: "0",
      city_id: city.id,
      street_id: street.id,
      house_id: house.id,
      city: "",
      street: "",
      house: "",
      _triggering_element_name: "search",
      _triggering_element_value: (searchBtn && searchBtn.value) || "Показати",
      _drupal_ajax: "1",
    };
    Object.entries(extra).forEach(([k, v]) => body.set(k, v));

    const action = new URL(form.getAttribute("action") || location.pathname, location.href);
    action.searchParams.set("ajax_form", "1");
    action.searchParams.set("_wrapper_format", "drupal_ajax");
    const cmds = commands(
      await fetchText(action.toString(), {
        method: "POST",
        body,
        headers: { Accept: "application/json, text/javascript, */*; q=0.01", "X-Requested-With": "XMLHttpRequest" },
      }),
    );
    const html = (Array.isArray(cmds) ? cmds : [])
      .map((c) => (c && typeof c.data === "string" ? c.data : ""))
      .filter((d) => d.includes("disconnection-detailed-table"))
      .join("\n");
    if (!html) throw new Error("сайт не повернув таблицю графіка");
    const doc = new DOMParser().parseFromString(html, "text/html");
    if (!report(doc, "auto")) throw new Error("таблицю не вдалося розібрати");
    return true;
  }

  // ---------- small helper bar on the page ----------

  let bar = null;
  function makeBar() {
    const el = document.createElement("div");
    el.setAttribute("style", "position:fixed;z-index:2147483647;right:16px;bottom:16px;max-width:380px;padding:12px 14px;border-radius:12px;background:#2b2926;color:#f3f1ec;font:500 13px/1.4 system-ui,sans-serif;box-shadow:0 6px 18px rgba(0,0,0,.35)");
    const title = document.createElement("div");
    title.textContent = "LightOFF";
    title.setAttribute("style", "font-weight:700;margin-bottom:4px;color:#ff6a33");
    const text = document.createElement("div");
    const btn = document.createElement("button");
    btn.textContent = "Зчитати графік";
    btn.setAttribute("style", "margin-top:8px;height:30px;padding:0 12px;border:0;border-radius:8px;background:#ff5a1f;color:#1f1e1c;font:600 13px system-ui,sans-serif;cursor:pointer");
    btn.onclick = () => {
      lastSent = "";
      if (!report(document, "page")) bar.say("Таблиці графіка на сторінці ще немає. Введіть адресу й натисніть «Показати».");
    };
    el.append(title, text, btn);
    document.body.appendChild(el);
    return { say: (s) => (text.textContent = s) };
  }

  function watch() {
    let timer = 0;
    new MutationObserver(() => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        if (document.querySelector(".disconnection-detailed-table-container")) report(document, "page");
      }, 600);
    }).observe(document.body, { childList: true, subtree: true });
  }

  function isChallenge() {
    const t = (document.title || "") + " " + (document.body ? document.body.innerText.slice(0, 400) : "");
    return /just a moment|трохи зачекайте|checking your browser|перевірка безпеки|verify you are human|підтвердьте, що ви людина/i.test(t);
  }

  function start() {
    if (isChallenge()) {
      send("status", { stage: "challenge" });
      return;
    }
    const form = document.querySelector("form.disconnection-detailed-search-form") || document.querySelector('form[id*="disconnection"]');
    if (!form) {
      send("status", { stage: "no_form", title: document.title });
      if (document.body) watch();
      return;
    }
    bar = makeBar();
    watch();
    if (document.querySelector(".disconnection-detailed-table-container") && report(document, "page")) return;
    if (CFG.auto && CFG.city && CFG.street && CFG.house) {
      bar.say("Шукаю графік для: " + CFG.city + ", " + CFG.street + ", " + CFG.house + "…");
      send("status", { stage: "searching" });
      autoSearch(form).catch((e) => {
        const msg = String((e && e.message) || e);
        bar.say("Автоматично не вийшло (" + msg + "). Введіть адресу й натисніть «Показати» — графік зчитається сам.");
        send("status", { stage: "manual", error: msg });
      });
    } else {
      bar.say("Введіть адресу й натисніть «Показати» — графік зчитається сам.");
      send("status", { stage: "manual" });
    }
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start);
  else start();
})();
