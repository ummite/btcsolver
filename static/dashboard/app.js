/**
 * BTC Solver — UI orientée "trouver des clés avec solde"
 */
(() => {
  "use strict";

  const $ = (id) => document.getElementById(id);
  const FOUND_KEY = "btcsolver_found_keys_v1";
  /** Explorateur public — adresse BTC uniquement, JAMAIS de clé privée dans l’URL */
  const BLOCKCHAIN_EXPLORER_ADDR =
    "https://www.blockchain.com/explorer/addresses/btc/";
  let lastHits = [];

  /** Block lag → human-readable time estimate (10 min/block average) */
  function formatLagTime(blocks) {
    if (!blocks || blocks <= 0) return "";
    const hours = (blocks * 10) / 60;
    if (hours >= 24) return ` (~${formatNumber(hours)} h / ~${(hours / 24).toFixed(1)} j)`;
    return ` (~${hours.toFixed(1)} h)`;
  }

  /** Compact (K/M/G) — pour débits, hits, etc. PAS pour hauteurs de bloc. */
  function formatNumber(n) {
    if (n == null || Number.isNaN(n)) return "—";
    if (n >= 1e12) return (n / 1e12).toFixed(2) + " T";
    if (n >= 1e9) return (n / 1e9).toFixed(2) + " G";
    if (n >= 1e6) return (n / 1e6).toFixed(2) + " M";
    if (n >= 1e3) return (n / 1e3).toFixed(1) + " K";
    return String(Math.round(Number(n)));
  }

  /**
   * Entier exact (hauteurs de bloc UTXO / Core) — jamais de raccourci K/M.
   * Espaces fins fr-FR pour lisibilité : 935000 → "935 000"
   */
  function formatHeight(n) {
    if (n == null || n === "" || Number.isNaN(Number(n))) return "—";
    return Math.round(Number(n)).toLocaleString("fr-FR");
  }

  function formatDuration(seconds) {
    if (seconds == null) return "—";
    const s = Math.floor(seconds);
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    if (h > 0) return `${h}h ${String(m).padStart(2, "0")}m`;
    if (m > 0) return `${m}m ${String(sec).padStart(2, "0")}s`;
    return `${sec}s`;
  }

  function esc(s) {
    return String(s ?? "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function toast(msg, type = "") {
    const el = $("toast");
    if (!el) return;
    el.textContent = msg;
    el.className = "toast" + (type ? " " + type : "");
    clearTimeout(toast._t);
    toast._t = setTimeout(() => el.classList.add("hidden"), 4500);
  }

  /** Bouton copier à droite d'une valeur (adresse / hex). */
  function copyBtn(text, label = "Copier", kind = "text") {
    if (!text || text === "—" || text === "error") return "";
    return `<button type="button" class="btn btn-ghost btn-sm btn-copy-inline" data-copy="${esc(
      text
    )}" data-copy-kind="${esc(kind)}" title="${esc(label)}">${esc(label)}</button>`;
  }

  async function copyTextToClipboard(t) {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(t);
      return;
    }
    const ta = document.createElement("textarea");
    ta.value = t;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    ta.remove();
  }

  /** Dérive la pub compressée depuis une priv (API locale). */
  async function derivePubkey(priv) {
    const hex = String(priv || "")
      .trim()
      .replace(/^0x/i, "")
      .toLowerCase();
    if (!/^[0-9a-f]{64}$/.test(hex)) return null;
    const r = await api("/api/keys/pubkeys", {
      method: "POST",
      body: JSON.stringify({ privkey_hex: hex }),
    });
    const p = r.pubkeys?.[hex] || r.results?.[0];
    if (!p || p.error) return null;
    return {
      pubkey_hex: p.pubkey_hex || p.compressed || "",
      pubkey_uncompressed_hex: p.pubkey_uncompressed_hex || p.uncompressed || "",
    };
  }

  function wireCopyButtons(root) {
    if (!root) return;
    root.querySelectorAll("[data-copy]").forEach((b) => {
      b.addEventListener("click", async (e) => {
        e.preventDefault();
        e.stopPropagation();
        let t = b.getAttribute("data-copy") || "";
        const kind = b.getAttribute("data-copy-kind") || "text";
        // Pub hex : uniquement format court compressé (02/03), jamais uncomp 04…
        if (kind === "pub" && t && !/^0[23]/i.test(t.trim())) {
          t = "";
        }
        // Si pub vide : dériver à la volée depuis data-priv
        if (kind === "pub" && !t) {
          const priv = b.getAttribute("data-priv") || "";
          if (!priv) {
            toast("No private key to derive public key", "error");
            return;
          }
          b.disabled = true;
          try {
            const d = await derivePubkey(priv);
            if (!d?.pubkey_hex) {
              toast("Cannot derive public key", "error");
              return;
            }
            t = d.pubkey_hex;
            b.setAttribute("data-copy", t);
            // Persiste dans le coffre local
            const list = loadFound();
            const pl = priv.toLowerCase();
            for (const m of list) {
              const mp = String(m.privkey_hex || m.key_hex || "").toLowerCase();
              if (mp === pl) {
                m.pubkey_hex = d.pubkey_hex;
                m.pubkey_uncompressed_hex = d.pubkey_uncompressed_hex;
              }
            }
            saveFound(list);
            // Met à jour la ligne affichée si présente
            const row = b.closest(".match-item")?.querySelector("[data-pub-value]");
            if (row) row.textContent = t;
          } catch (err) {
            toast(err.message || "Public key derivation error", "error");
            return;
          } finally {
            b.disabled = false;
          }
        }
        if (!t) {
          toast("Nothing to copy", "error");
          return;
        }
        const okMsg =
          kind === "pub"
            ? "Clé publique copiée"
            : kind === "priv"
              ? "Clé privée copiée"
              : kind === "addr"
                ? "Adresse copiée"
                : "Copié";
        try {
          await copyTextToClipboard(t);
          toast(okMsg, "success");
        } catch (_) {
          toast("Copie impossible", "error");
        }
      });
    });
  }

  /**
   * Ligne label + valeur + bouton copier à droite.
   * kind: "pub" | "priv" | "addr" | "text"
   * force: affiche la ligne même si value vide (ex: pub à dériver)
   */
  function rowCopy(label, value, kind = "text", opts = {}) {
    const force = !!opts.force;
    if (!force && (value == null || value === "")) return "";
    const full = value == null || value === "" ? "" : String(value);
    // Affichage court pour les longs hex / adresses — copie = valeur complète
    const display =
      full === ""
        ? "—"
        : opts.short !== false && full.length > 28
          ? shortDisplay(full, opts.head || 12, opts.tail || 10)
          : full;
    const btnLabel =
      kind === "pub"
        ? "Copier pub"
        : kind === "priv"
          ? "Copier priv"
          : kind === "addr"
            ? "Copier addr"
            : "Copier";
    const privAttr =
      kind === "pub" && opts.priv
        ? ` data-priv="${esc(opts.priv)}"`
        : "";
    const pubMark = kind === "pub" ? " data-pub-value" : "";
    let btn = "";
    if (full) {
      btn = `<button type="button" class="btn btn-ghost btn-sm btn-copy-inline" data-copy="${esc(
        full
      )}" data-copy-kind="${esc(kind)}"${privAttr} title="${esc(full)}">${esc(btnLabel)}</button>`;
    } else if (kind === "pub" && opts.priv) {
      btn = `<button type="button" class="btn btn-ghost btn-sm btn-copy-inline" data-copy="" data-copy-kind="pub"${privAttr} title="Dériver puis copier la pub compressée">Copier pub</button>`;
    }
    // Lien explorateur uniquement pour les adresses publiques (jamais priv / pub hex)
    const explorer =
      kind === "addr" && isBtcPublicAddress(full)
        ? explorerLinkHtml(full, "Vérifier solde")
        : "";
    return `<div class="result-row result-row-copy">
      <span class="label">${esc(label)}</span>
      <span class="value hex-full"${pubMark} title="${esc(full || "")}">${esc(display)}</span>
      ${btn}
      ${explorer}
    </div>`;
  }

  /** Affichage court (head…tail) — la copie garde toujours la valeur complète. */
  function shortDisplay(s, head = 10, tail = 8) {
    const t = String(s || "");
    if (t.length <= head + tail + 1) return t;
    return t.slice(0, head) + "…" + t.slice(-tail);
  }

  /** Vraie adresse BTC publique (legacy / P2SH / bech32) — pas de priv / WIF / hex. */
  function isBtcPublicAddress(a) {
    const s = String(a || "").trim();
    if (!s || s === "—") return false;
    // Refuse explicitement tout ce qui ressemble à une clé privée
    if (/^[0-9a-fA-F]{64}$/.test(s)) return false;
    if (/^[5KL][1-9A-HJ-NP-Za-km-z]{50,51}$/.test(s)) return false; // WIF
    return /^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$|^bc1[a-z0-9]{25,90}$/i.test(s);
  }

  /**
   * URL blockchain.com pour une adresse publique uniquement.
   * Ne jamais y coller priv / WIF / seed.
   */
  function explorerAddrUrl(addr) {
    if (!isBtcPublicAddress(addr)) return "";
    return BLOCKCHAIN_EXPLORER_ADDR + encodeURIComponent(String(addr).trim());
  }

  /** Lien « vérifier solde » — target=_blank, rel=noopener (adresse publique seule). */
  function explorerLinkHtml(addr, label = "Vérifier solde") {
    const url = explorerAddrUrl(addr);
    if (!url) return "";
    return `<a class="btn btn-explorer btn-sm" href="${esc(url)}" target="_blank" rel="noopener noreferrer" title="Ouvre blockchain.com avec l’adresse PUBLIQUE uniquement — aucune clé privée n’est envoyée">${esc(label)} ↗</a>`;
  }

  /**
   * Adresse publique « format court » pour explorers :
   * priorité à l’adresse du hit, sinon la plus courte parmi legacy / segwit / wrapped / taproot.
   * (1… et 3… sont souvent plus courts que bc1p…)
   */
  function preferredShortAddress(m) {
    const hit = (m.address || "").trim();
    const pool = [
      hit,
      m.addresses?.legacy,
      m.addresses?.segwit,
      m.addresses?.wrapped,
      m.addresses?.taproot,
    ]
      .map((a) => (a || "").trim())
      .filter((a) => a && a !== "—");
    if (!pool.length) return "";
    // Dédup
    const uniq = [...new Set(pool)];
    // Si le hit est une vraie adresse BTC, on le garde (c’est celle du solde trouvé)
    if (hit && isBtcPublicAddress(hit)) {
      return hit;
    }
    // Sinon la plus courte (format « court ») parmi les adresses valides
    const valid = uniq.filter(isBtcPublicAddress);
    if (!valid.length) return uniq[0] || "";
    return valid.slice().sort((a, b) => a.length - b.length || a.localeCompare(b))[0];
  }

  /** Force pubkey compressée (02/03) — jamais l’uncomp 04… longue. */
  function ensureCompressedPub(pub) {
    const p = String(pub || "").trim().toLowerCase();
    if (/^0[23][0-9a-f]{64}$/.test(p)) return p;
    return ""; // uncomp ou invalide → à dériver
  }

  /**
   * Barre coffre : PRIV (rouge) + ADRESSE publique courte (vert) + PUB compressée (vert clair).
   */
  function vaultKeyActions(priv, addr, pub) {
    const p = priv || "";
    const a = addr || "";
    const u = ensureCompressedPub(pub) || pub || "";
    const explor = explorerLinkHtml(a, "Vérifier solde on-chain");
    return `<div class="vault-key-actions">
      <button type="button" class="btn btn-copy-priv-lg" data-copy="${esc(p)}" data-copy-kind="priv" ${
        p ? "" : "disabled"
      } title="Copier la clé privée (hex 64) — rester en local, ne jamais coller dans un navigateur web">⬛ Copier PRIV</button>
      <button type="button" class="btn btn-copy-addr-lg" data-copy="${esc(a)}" data-copy-kind="addr" ${
        a ? "" : "disabled"
      } title="Copier l’adresse publique (format court pour explorers)">🟩 Copier ADDR</button>
      <button type="button" class="btn btn-copy-pub-lg" data-copy="${esc(u)}" data-copy-kind="pub" data-priv="${esc(
      p
    )}" ${p || u ? "" : "disabled"} title="Clé publique compressée courte (02/03…, 33 octets)">Copier PUB hex</button>
      ${explor || `<span class="hint" style="align-self:center">pas d’adresse publique pour explorer</span>`}
    </div>`;
  }

  /** Bip navigateur (Web Audio) — jackpot 3 notes. Throttle 1.2s. */
  let _lastHitBeep = 0;
  let _audioCtx = null;
  function playHitBeep() {
    const now = Date.now();
    if (now - _lastHitBeep < 1200) return;
    _lastHitBeep = now;
    try {
      const AC = window.AudioContext || window.webkitAudioContext;
      if (!AC) return;
      if (!_audioCtx) _audioCtx = new AC();
      const ctx = _audioCtx;
      if (ctx.state === "suspended") ctx.resume();
      const freqs = [880, 1175, 1568, 1568];
      freqs.forEach((freq, i) => {
        const o = ctx.createOscillator();
        const g = ctx.createGain();
        o.type = "square";
        o.frequency.value = freq;
        o.connect(g);
        g.connect(ctx.destination);
        const t0 = ctx.currentTime + i * 0.16;
        g.gain.setValueAtTime(0.0001, t0);
        g.gain.exponentialRampToValueAtTime(0.18, t0 + 0.02);
        g.gain.exponentialRampToValueAtTime(0.0001, t0 + 0.14);
        o.start(t0);
        o.stop(t0 + 0.15);
      });
    } catch (_) {
      try {
        // fallback très basique
        if (typeof window.speechSynthesis !== "undefined") {
          /* ignore */
        }
      } catch (_) {}
    }
  }

  /** Suivi des hits dict pour bip seulement sur nouveaux soldes */
  let _dictLastMatches = 0;

  function setMsg(id, text, type = "") {
    const el = $(id);
    if (!el) return;
    el.textContent = text || "";
    el.className = "form-msg" + (type ? " " + type : "");
  }

  function setText(id, v) {
    const el = $(id);
    if (el) el.textContent = v == null || v === "" ? "—" : String(v);
  }

  function setPill(id, text, kind, title) {
    const el = $(id);
    if (!el) return;
    el.textContent = text;
    el.className = "pill" + (kind ? " " + kind : "");
    // Full text on hover (pill may be truncated to fixed width)
    el.title = title != null && title !== "" ? String(title) : String(text || "");
  }

  /** Compact fixed-width number: always like "199K" / "1.2M" (no spaces) */
  function formatCompact(n) {
    if (n == null || Number.isNaN(Number(n))) return "—";
    const x = Number(n);
    if (x >= 1e12) return (x / 1e12).toFixed(1) + "T";
    if (x >= 1e9) return (x / 1e9).toFixed(1) + "G";
    if (x >= 1e6) return (x / 1e6).toFixed(2) + "M";
    if (x >= 1e3) return (x / 1e3).toFixed(0) + "K";
    return String(Math.round(x));
  }

  async function api(path, options = {}) {
    const ctrl = new AbortController();
    const timeoutMs = options.timeoutMs || 20000;
    const t = setTimeout(() => ctrl.abort(), timeoutMs);
    try {
      const { timeoutMs: _t, headers: extraHeaders, ...fetchOpts } = options;
      const res = await fetch(path, {
        ...fetchOpts,
        headers: { "Content-Type": "application/json", ...(extraHeaders || {}) },
        signal: ctrl.signal,
      });
      const text = await res.text();
      let data = {};
      try {
        data = text ? JSON.parse(text) : {};
      } catch (pe) {
        throw new Error(`JSON invalide (${path}): ${text.slice(0, 120)}`);
      }
      // Only treat TOP-LEVEL API failures as errors (not nested bitcoind.message)
      if (!res.ok) {
        throw new Error(data.error || data.message || `HTTP ${res.status} ${path}`);
      }
      if (data.success === false) {
        throw new Error(data.error || data.message || "échec API");
      }
      return data;
    } finally {
      clearTimeout(t);
    }
  }

  // ── Tabs ────────────────────────────────────────────────────────────────
  document.querySelectorAll(".tab").forEach((btn) => {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".tab").forEach((b) => b.classList.remove("active"));
      document.querySelectorAll(".panel").forEach((p) => p.classList.remove("active"));
      btn.classList.add("active");
      const panel = $("panel-" + btn.dataset.tab);
      if (panel) panel.classList.add("active");
      if (btn.dataset.tab === "found") void renderFound();
    });
  });

  // ── Clock ───────────────────────────────────────────────────────────────
  function tickClock() {
    const now = new Date();
    if ($("clock"))
      $("clock").textContent = now.toLocaleTimeString("fr-FR", { hour12: false });
    if ($("clockUtc"))
      $("clockUtc").textContent = now.toLocaleTimeString("fr-FR", { hour12: false, timeZone: "UTC" }) + " (UTC)";
    updateBlockTime();
  }
  setInterval(tickClock, 1000);
  tickClock();

  // ── Block time display ──────────────────────────────────────────────────
  function updateBlockTime() {
    const el = $("blockTime");
    if (!el) return;
    const btc = window.__lastBtc;
    if (!btc || !btc.block_time) {
      el.innerHTML = "bloc — · --:--:--";
      return;
    }
    const height = btc.blocks != null ? formatHeight(btc.blocks) : "—";
    const blockDt = new Date(Number(btc.block_time) * 1000);
    const blockStr = blockDt.toLocaleTimeString("fr-FR", { hour12: false });
    const now = Date.now() / 1000;
    const lagSec = now - Number(btc.block_time);
    let lagStr = "";
    if (lagSec < 60) lagStr = `il y a ${Math.round(lagSec)}s`;
    else if (lagSec < 3600) lagStr = `il y a ${Math.round(lagSec / 60)}min`;
    else lagStr = `il y a ${Math.round(lagSec / 3600)}h${Math.round((lagSec % 3600) / 60)}`;
    el.innerHTML = `<span class="bt-height">${height}</span> · <span class="bt-time">${blockStr}</span> <span class="bt-lag">(${lagStr})</span>`;
  }

  // ── Found vault ─────────────────────────────────────────────────────────
  function loadFound() {
    try {
      return JSON.parse(localStorage.getItem(FOUND_KEY) || "[]");
    } catch {
      return [];
    }
  }
  function saveFound(list) {
    localStorage.setItem(FOUND_KEY, JSON.stringify(list));
  }
  function addFound(entries) {
    const list = loadFound();
    const seen = new Set(list.map((x) => x.privkey_hex + "|" + (x.address || "")));
    let n = 0;
    for (const e of entries) {
      const k = (e.privkey_hex || "") + "|" + (e.address || e.addresses?.legacy || "");
      if (seen.has(k)) continue;
      seen.add(k);
      list.unshift({ ...e, saved_at: new Date().toISOString() });
      n++;
    }
    saveFound(list.slice(0, 500));
    return n;
  }
  function normalizeVaultEntry(m) {
    // Unifie les noms de champs (export / archive / hits)
    if (!m.privkey_hex) {
      m.privkey_hex = m.key_hex || m.priv || m.private_key || "";
    }
    if (!m.pubkey_hex) {
      m.pubkey_hex = m.public_key || m.pub || m.compressed_pubkey || "";
    }
    return m;
  }

  /** Remplit pubkey_hex manquantes via le backend (priv → pub SEC1). */
  async function enrichFoundPubkeys() {
    const list = loadFound().map(normalizeVaultEntry);
    const need = [
      ...new Set(
        list
          .filter((m) => m.privkey_hex && !m.pubkey_hex)
          .map((m) => String(m.privkey_hex).replace(/^0x/i, "").toLowerCase())
          .filter((h) => /^[0-9a-f]{64}$/.test(h))
      ),
    ];
    if (!need.length) {
      saveFound(list);
      return false;
    }
    try {
      const r = await api("/api/keys/pubkeys", {
        method: "POST",
        body: JSON.stringify({ privkeys: need }),
      });
      const map = r.pubkeys || {};
      let changed = false;
      for (const m of list) {
        const k = String(m.privkey_hex || "")
          .replace(/^0x/i, "")
          .toLowerCase();
        const p = map[k];
        if (!p) continue;
        if (!m.pubkey_hex && (p.pubkey_hex || p.compressed)) {
          m.pubkey_hex = p.pubkey_hex || p.compressed;
          changed = true;
        }
        if (!m.pubkey_uncompressed_hex && (p.pubkey_uncompressed_hex || p.uncompressed)) {
          m.pubkey_uncompressed_hex = p.pubkey_uncompressed_hex || p.uncompressed;
          changed = true;
        }
      }
      saveFound(list);
      return changed;
    } catch (_) {
      saveFound(list);
      return false;
    }
  }

  async function renderFound() {
    const box = $("foundList");
    if (!box) return;
    box.innerHTML = `<p class="hint">Préparation des clés publiques…</p>`;
    await enrichFoundPubkeys();
    const list = loadFound().map(normalizeVaultEntry);
    if (!list.length) {
      box.innerHTML =
        "Aucune clé sauvée. Fais un test qui trouve un solde, puis « Sauver les hits ». Les pubkeys sont dérivées auto pour coller sur un explorer.";
      return;
    }
    box.innerHTML = list
      .map((m, i) => {
        const bal = m.total_balance_btc ?? m.value_btc ?? 0;
        const addrShort = preferredShortAddress(m);
        const priv = m.privkey_hex || m.key_hex || "";
        // Pub = uniquement compressée (format court 02/03…), jamais uncomp 04…
        let pub = ensureCompressedPub(m.pubkey_hex);
        if (!pub && m.pubkey_hex && String(m.pubkey_hex).startsWith("04")) {
          pub = ""; // forcer re-dérivation compressée au clic
        }
        return `<div class="match-item">
          <div class="result-balance">${bal} BTC</div>
          <div class="method-tag">${esc(m.method || m.input_format || "")}</div>
          ${vaultKeyActions(priv, addrShort, pub)}
          <div class="result-row"><span class="label">entrée</span><span class="value">${esc(m.input || m.phrase || "")}</span></div>
          ${rowCopy("adresse publique", addrShort, "addr", { force: true, short: false })}
          ${rowCopy("priv hex", priv, "priv", { force: true, head: 10, tail: 8 })}
          ${rowCopy("pub hex (comp courte)", pub, "pub", { force: true, priv, head: 10, tail: 8 })}
          ${m.addresses?.legacy && m.addresses.legacy !== addrShort ? rowCopy("legacy", m.addresses.legacy, "addr", { short: false }) : ""}
          ${m.addresses?.segwit && m.addresses.segwit !== addrShort ? rowCopy("segwit", m.addresses.segwit, "addr", { short: false }) : ""}
          ${m.addresses?.wrapped && m.addresses.wrapped !== addrShort ? rowCopy("wrapped", m.addresses.wrapped, "addr", { short: false }) : ""}
          ${m.addresses?.taproot && m.addresses.taproot !== addrShort ? rowCopy("taproot", m.addresses.taproot, "addr", { short: false }) : ""}
          <div class="result-row"><span class="label">sauvé</span><span class="value">${esc(m.saved_at || "")}</span></div>
          <button type="button" class="btn btn-ghost btn-sm" data-rm="${i}">Retirer</button>
        </div>`;
      })
      .join("");
    wireCopyButtons(box);
    box.querySelectorAll("[data-rm]").forEach((b) =>
      b.addEventListener("click", async () => {
        const list = loadFound();
        list.splice(Number(b.getAttribute("data-rm")), 1);
        saveFound(list);
        await renderFound();
      })
    );
  }

  // ── Methods presets ─────────────────────────────────────────────────────
  function setHuntMethods(max) {
    const ids = [
      "optSha256",
      "optDouble",
      "optMd5",
      "optRevChars",
      "optRevWords",
      "optLower",
      "optUpper",
      "optNoSpace",
      "optStripSym",
      "optSuffix",
      "optPrefix",
      "optBipAll",
    ];
    ids.forEach((id) => {
      if ($(id)) $(id).checked = !!max;
    });
    if ($("optSha256")) $("optSha256").checked = true;
    if ($("optLower")) $("optLower").checked = true;
    if ($("optStripSym")) $("optStripSym").checked = true;
    if ($("bipCount")) $("bipCount").value = max ? 10 : 3;
    if (!max) {
      if ($("optDouble")) $("optDouble").checked = false;
      if ($("optMd5")) $("optMd5").checked = false;
      if ($("optUpper")) $("optUpper").checked = false;
      if ($("optNoSpace")) $("optNoSpace").checked = false;
      if ($("optSuffix")) $("optSuffix").checked = false;
      if ($("optPrefix")) $("optPrefix").checked = false;
    }
  }
  function setDictMethods(max) {
    ["dSha256", "dDouble", "dMd5", "dRevC", "dRevW", "dLower", "dStripSym", "dNoSpace", "dSuffix", "dPrefix"].forEach(
      (id) => {
        if ($(id)) $(id).checked = !!max;
      }
    );
    if ($("dSha256")) $("dSha256").checked = true;
    if ($("dLower")) $("dLower").checked = true;
    if ($("dStripSym")) $("dStripSym").checked = true;
    if (!max) {
      ["dDouble", "dMd5", "dNoSpace", "dSuffix", "dPrefix"].forEach((id) => {
        if ($(id)) $(id).checked = false;
      });
    }
  }
  function brainOpts() {
    return {
      sha256: $("optSha256")?.checked ?? true,
      double_sha256: $("optDouble")?.checked ?? false,
      md5_padded: $("optMd5")?.checked ?? false,
      reverse_chars: $("optRevChars")?.checked ?? true,
      reverse_words: $("optRevWords")?.checked ?? true,
      lowercase: $("optLower")?.checked ?? true,
      uppercase: $("optUpper")?.checked ?? false,
      no_spaces: $("optNoSpace")?.checked ?? false,
      strip_symbols: $("optStripSym")?.checked ?? false,
      common_suffixes: $("optSuffix")?.checked ?? false,
      common_prefixes: $("optPrefix")?.checked ?? false,
      bip39_all_paths: $("optBipAll")?.checked ?? true,
      bip39_address_count: parseInt($("bipCount")?.value || "10", 10) || 10,
    };
  }

  $("btnModeMax")?.addEventListener("click", () => {
    setHuntMethods(true);
    toast("Mode MAX activé", "success");
  });
  $("btnModeFast")?.addEventListener("click", () => {
    setHuntMethods(false);
    toast("Mode rapide", "");
  });
  $("btnDictMax")?.addEventListener("click", () => setDictMethods(true));
  $("btnDictFast")?.addEventListener("click", () => setDictMethods(false));
  $("btnClearKey")?.addEventListener("click", () => {
    if ($("keyInput")) $("keyInput").value = "";
    if ($("keyResult")) $("keyResult").innerHTML = "Aucun test pour l’instant.";
  });

  // ── Key check (single or multi-line batch) ──────────────────────────────
  function renderKeyResults(data) {
    const box = $("keyResult");
    if (!box) return;
    if (data.error) {
      box.innerHTML = `<div class="result-row"><span class="label">Erreur</span><span class="value">${esc(data.error)}</span></div>`;
      return;
    }
    const results = data.results || [];
    lastHits = results.filter((r) => (r.total_balance_sats || 0) > 0);
    if (!results.length) {
      box.innerHTML = `<p class="hint">0 hit affiché (candidats: ${data.candidates || 0}). Décoche « Seulement soldes > 0 » pour voir les adresses à 0.</p>`;
      return;
    }
    let html = `<p class="hint">${data.candidates || results.length} candidats · total ${data.total_balance_btc || 0} BTC · hits ${lastHits.length}</p>`;
    for (const r of results.slice(0, 60)) {
      const bal = r.total_balance_sats > 0;
      const priv = r.privkey_hex || "";
      const pub = r.pubkey_hex || "";
      html += `<div class="match-item" style="${bal ? "" : "background:var(--bg-input);border-color:var(--border)"}">`;
      html += `<div class="method-tag">${esc(r.method || r.input_format)}</div>`;
      if (bal)
        html += `<div class="result-balance">${r.total_balance_btc} BTC (${formatNumber(r.total_balance_sats)} sats)</div>`;
      else html += `<div class="type">0 BTC</div>`;
      html += `<div class="result-row"><span class="label">entrée</span><span class="value hex-full">${esc(r.input)}</span></div>`;
      html += rowCopy("priv hex (64)", priv, "priv");
      // Clé publique compressée (hex) — bouton dédié « Copier pub » (pas la privée)
      html += rowCopy("clé publique (pub hex)", pub || "", "pub");
      if (r.addresses) {
        html += rowCopy("legacy", r.addresses.legacy, "addr");
        html += rowCopy("segwit", r.addresses.segwit, "addr");
        if (r.addresses.wrapped) html += rowCopy("wrapped", r.addresses.wrapped, "addr");
        html += rowCopy("taproot", r.addresses.taproot, "addr");
      }
      if (r.matches?.length) {
        for (const m of r.matches) {
          html += `<div class="type">${esc(m.address_type)} · ${m.value_btc} BTC</div>`;
          html += rowCopy("adresse", m.address, "addr");
        }
      }
      html += `</div>`;
    }
    box.innerHTML = html;
    box.className = "key-result" + (data.total_balance_sats > 0 ? " has-balance" : "");
    wireCopyButtons(box);
  }

  $("keyForm")?.addEventListener("submit", async (e) => {
    e.preventDefault();
    const raw = ($("keyInput")?.value || "").trim();
    if (!raw) return setMsg("keyMsg", "Vide", "error");
    const lines = raw
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter((l) => l && !l.startsWith("#"));
    setMsg("keyMsg", lines.length > 1 ? `Lot de ${lines.length}…` : "Test…");
    try {
      let data;
      if (lines.length === 1) {
        data = await api("/api/keys/check", {
          method: "POST",
          body: JSON.stringify({
            key: lines[0],
            format: $("keyFormat")?.value || null,
            passphrase: $("passphrase")?.value || null,
            options: brainOpts(),
            matches_only: $("matchesOnly")?.checked ?? true,
          }),
        });
      } else {
        data = await api("/api/keys/batch", {
          method: "POST",
          body: JSON.stringify({
            keys: lines,
            format: $("keyFormat")?.value || null,
            passphrase: $("passphrase")?.value || null,
            options: brainOpts(),
          }),
        });
        // batch already filters to balance>0 only in backend for each derived
      }
      if (data.error) throw new Error(data.error);
      renderKeyResults(data);
      const hits = (data.results || []).filter((r) => (r.total_balance_sats || 0) > 0);
      setMsg(
        "keyMsg",
        `${data.candidates || data.count || 0} candidats · ${hits.length} hit(s)`,
        hits.length ? "success" : ""
      );
      if (hits.length) {
        flashHitScreen(hits.length, true);
        toast(`HIT: ${data.total_balance_btc} BTC`, "success");
        addFound(hits);
        renderFound();
      }
    } catch (err) {
      setMsg("keyMsg", err.message, "error");
      toast(err.message, "error");
    }
  });

  $("btnSaveHits")?.addEventListener("click", () => {
    const n = addFound(lastHits);
    toast(n ? `${n} clé(s) ajoutée(s) au coffre` : "Rien de nouveau à sauver", n ? "success" : "");
    renderFound();
  });
  $("btnExportFound")?.addEventListener("click", () => {
    const blob = new Blob([JSON.stringify(loadFound(), null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `btcsolver-found-${Date.now()}.json`;
    a.click();
  });
  $("btnClearFound")?.addEventListener("click", () => {
    if (confirm("Vider le coffre local ?")) {
      saveFound([]);
      renderFound();
    }
  });

  // ── Dict ────────────────────────────────────────────────────────────────
  async function loadCorpora() {
    try {
      const r = await api("/api/dict/corpora");
      const sel = $("dictCorpus");
      if (!sel) return;
      for (const c of r.corpora || []) {
        const o = document.createElement("option");
        o.value = c.path;
        o.textContent = `${c.name} (${c.size_mb} Mo)`;
        sel.appendChild(o);
      }
    } catch (_) {}
  }

  function updateDict(d) {
    if (!d) return;
    const mf = Number(d.matches_found || 0);
    if (mf > _dictLastMatches) {
      flashHitScreen(mf, _dictLastMatches <= 0);
    }
    if (!d.running && d.done) _dictLastMatches = 0;
    else _dictLastMatches = mf;

    setText("dictTested", formatNumber(d.keys_tested));
    setText("dictVariants", formatNumber(d.variants_total));
    // Vitesse totale + détail par GPU / CPU
    let rateStr = d.keys_per_sec ? formatNumber(d.keys_per_sec) + "/s" : "—";
    setText("dictRate", rateStr);
    const parts = [];
    if (Array.isArray(d.gpu_rates) && d.gpu_rates.length) {
      for (const g of d.gpu_rates) {
        parts.push(
          `GPU${g.id}: ${formatNumber(g.keys_per_sec || 0)}/s (${formatNumber(g.keys_tested || 0)})`
        );
      }
    }
    if (d.cpu_threads != null || d.cpu_keys_per_sec != null) {
      parts.push(
        `CPU×${d.cpu_threads || "?"} : ${formatNumber(d.cpu_keys_per_sec || 0)}/s (${formatNumber(d.cpu_keys_tested || 0)})`
      );
    }
    if ($("dictRateDetail")) {
      $("dictRateDetail").textContent = parts.length
        ? parts.join(" · ")
        : d.engine || "—";
    }
    setText("dictMatches", formatNumber(mf));
    const pct = d.progress_pct || 0;
    if ($("dictProgress")) $("dictProgress").style.width = pct.toFixed(1) + "%";
    setText("dictProgressLabel", pct.toFixed(1) + "%");
    const eng = d.engine ? ` · ${d.engine}` : "";
    setText(
      "dictLast",
      (d.last_phrase ? "last: " + d.last_phrase : "—") + eng
    );
    if (d.matches?.length && $("dictResults")) {
      $("dictResults").innerHTML = d.matches
        .slice(0, 40)
        .map(
          (m) => `<div class="match-item">
          <div class="result-balance">${m.value_btc} BTC</div>
          <div class="method-tag">${esc(m.method)}</div>
          <div class="value">${esc(m.phrase)}</div>
          ${rowCopy("adresse", m.address, "addr")}
          ${m.pubkey_hex ? rowCopy("clé publique", m.pubkey_hex, "pub") : ""}
          ${rowCopy("priv hex", m.privkey_hex, "priv")}
        </div>`
        )
        .join("");
      wireCopyButtons($("dictResults"));
      // auto-save dict hits (incl. pubkeys pour le coffre / explorers)
      addFound(
        d.matches.map((m) => ({
          input: m.phrase,
          method: m.method,
          privkey_hex: m.privkey_hex,
          pubkey_hex: m.pubkey_hex || "",
          pubkey_uncompressed_hex: m.pubkey_uncompressed_hex || "",
          address: m.address,
          value_btc: m.value_btc,
          total_balance_btc: m.value_btc,
          total_balance_sats: m.value_sats,
        }))
      );
    }
  }

  function dictTokenCount() {
    const text = $("dictPhrases")?.value || "";
    const set = new Set();
    for (const line of text.split(/\r?\n/)) {
      for (const t of line.trim().split(/\s+/)) {
        if (t) set.add(t.toLowerCase());
      }
    }
    return set.size;
  }

  function estimateDictBag() {
    const n = Math.min(dictTokenCount(), parseInt($("dictMaxWords")?.value || "7", 10) || 7);
    const perms = !!$("dPerms")?.checked;
    const combos = !!$("dCombos")?.checked;
    const joins =
      ($("dJoinSpace")?.checked ? 1 : 0) + ($("dJoinNoSpace")?.checked ? 1 : 0) || 1;
    const el = $("dictPermEst");
    if (!el) return;
    if (!perms && !combos) {
      el.textContent = "— (options off : 1 phrase / ligne seulement)";
      return;
    }
    let count = 0;
    if (combos && perms) {
      // sum P(n,k)
      let p = 1;
      for (let k = 1; k <= n; k++) {
        p *= n - k + 1;
        count += p;
      }
    } else if (perms) {
      count = 1;
      for (let k = 2; k <= n; k++) count *= k;
    } else if (combos) {
      count = (1 << n) - 1;
    }
    count *= joins;
    const warn = count > 100000 ? " ⚠ volumineux" : count > 10000 ? " · moyen" : " · léger";
    el.textContent = `~${count.toLocaleString("fr-FR")} phrases base (n=${n} mots)${warn} + hashes`;
  }

  /** Estimation affixes tous-caractères (94 ASCII) par phrase de base. */
  function estimateAffix() {
    const el = $("dAffixEst");
    if (!el) return;
    const pre = Math.min(2, Math.max(0, parseInt($("dCharPrefixLen")?.value || "0", 10) || 0));
    const suf = Math.min(2, Math.max(0, parseInt($("dCharSuffixLen")?.value || "0", 10) || 0));
    if (pre === 0 && suf === 0) {
      el.textContent = "affixes : off";
      return;
    }
    const C = 94;
    const nPre = pre === 0 ? 1 : pre === 1 ? C : C * C;
    const nSuf = suf === 0 ? 1 : suf === 1 ? C : C * C;
    const perBase = nPre * nSuf;
    const note =
      perBase > 5e6
        ? " · très long (streaming, pas de plafond)"
        : perBase > 50000
          ? " · long (boucles streaming)"
          : " · ok";
    el.textContent = `affixes : ~${perBase.toLocaleString("fr-FR")} × bases × hashes / phrase (pref ${pre} × suf ${suf})${note}`;
  }

  ["dictPhrases", "dPerms", "dCombos", "dJoinSpace", "dJoinNoSpace", "dictMaxWords"].forEach(
    (id) => {
      $(id)?.addEventListener("input", estimateDictBag);
      $(id)?.addEventListener("change", estimateDictBag);
    }
  );
  ["dCharPrefixLen", "dCharSuffixLen"].forEach((id) => {
    $(id)?.addEventListener("change", estimateAffix);
  });
  estimateDictBag();
  estimateAffix();

  $("dictForm")?.addEventListener("submit", async (e) => {
    e.preventDefault();
    const body = {
      phrases: $("dictPhrases")?.value || "",
      corpus_path: $("dictCorpus")?.value || null,
      threads: parseInt($("dictThreads")?.value || "0", 10) || null,
      min_value: parseInt($("dictMin")?.value || "0", 10) || 0,
      word_permutations: !!$("dPerms")?.checked,
      word_combinations: !!$("dCombos")?.checked,
      join_with_space: !!$("dJoinSpace")?.checked,
      join_no_space: !!$("dJoinNoSpace")?.checked,
      max_perm_words: parseInt($("dictMaxWords")?.value || "7", 10) || 7,
      use_gpu: $("dUseGpu") ? !!$("dUseGpu").checked : true,
      // FULL VRAM: backend lit BTC_GPU_FULL via env n'est plus requis — flag API
      gpu_full: $("dGpuFull") ? !!$("dGpuFull").checked : true,
      options: {
        sha256: $("dSha256")?.checked ?? true,
        double_sha256: $("dDouble")?.checked ?? false,
        md5_padded: $("dMd5")?.checked ?? false,
        reverse_chars: $("dRevC")?.checked ?? true,
        reverse_words: $("dRevW")?.checked ?? true,
        lowercase: $("dLower")?.checked ?? true,
        uppercase: false,
        no_spaces: !!$("dNoSpace")?.checked,
        strip_symbols: !!$("dStripSym")?.checked,
        common_suffixes: $("dSuffix")?.checked ?? false,
        common_prefixes: $("dPrefix")?.checked ?? false,
        char_prefix_len: parseInt($("dCharPrefixLen")?.value || "0", 10) || 0,
        char_suffix_len: parseInt($("dCharSuffixLen")?.value || "0", 10) || 0,
        bip39_all_paths: false,
        bip39_address_count: 1,
      },
    };
    setMsg("dictMsg", "Démarrage…");
    try {
      await api("/api/dict/start", { method: "POST", body: JSON.stringify(body) });
      setMsg("dictMsg", "Scan lancé", "success");
      toast("Scan dictionnaire démarré", "success");
    } catch (err) {
      setMsg("dictMsg", err.message, "error");
      toast(err.message, "error");
    }
  });
  $("btnDictStop")?.addEventListener("click", async () => {
    await api("/api/dict/stop", { method: "POST", body: "{}" });
    setMsg("dictMsg", "Arrêt demandé");
  });

  // ── Status / Core ───────────────────────────────────────────────────────
  function updateStatusBar(s, health) {
    const running = !!(s?.running || s?.process_running);
    const rpc = !!s?.rpc_ok;
    const synced = !!s?.is_synced;
    const indexOk = !!(health?.index_loaded);
    const bruteOn = !!(health?.scan?.running || health?.scan_bg);
    const dictOn = !!health?.dict?.running;
    const anyScanOn = bruteOn || dictOn;

    // Aggregate GPU + CPU speeds from all scan sources (brute + dict)
    const scanData = health?.scan || {};
    const dictData = health?.dict || {};
    let gpuTotal = 0, cpuTotal = 0;
    const gpuParts = [];

    // Brute-force scan GPU rates
    if (Array.isArray(scanData.gpu_rates)) {
      for (const g of scanData.gpu_rates) {
        const r = Number(g.keys_per_sec || g.keys_per_sec_avg || 0);
        gpuTotal += r;
        gpuParts.push(`GPU${g.id}: ${formatCompact(r)}/s`);
      }
    }
    cpuTotal += Number(scanData.cpu_keys_per_sec || 0);

    // Dict scan GPU rates
    if (Array.isArray(dictData.gpu_rates)) {
      for (const g of dictData.gpu_rates) {
        const r = Number(g.keys_per_sec || g.keys_per_sec_avg || 0);
        gpuTotal += r;
        gpuParts.push(`GPU${g.id}: ${formatCompact(r)}/s (dict)`);
      }
    }
    cpuTotal += Number(dictData.cpu_keys_per_sec || 0);

    const totalRate = gpuTotal + cpuTotal;
    const totalTested = Math.max(dictData.keys_tested || 0, scanData.keys_tested || 0);

    // Pill: simple ON/OFF
    if (anyScanOn || totalRate > 0) {
      setPill("pillScan", "Scan ON", "ok", `${formatCompact(totalRate)}/s · ${formatNumber(totalTested)} testées`);
    } else {
      setPill("pillScan", "Scan OFF", "warn", "Scan de fond arrêté");
    }

    // Update "Scan en cours" tile with GPU/CPU breakdown
    if ($("infoScanRate")) {
      $("infoScanRate").textContent = totalRate > 0 ? `${formatCompact(totalRate)} /s` : "— /s";
    }
    if ($("infoScanTested")) {
      $("infoScanTested").textContent = totalTested > 0 ? `testées: ${formatNumber(totalTested)}` : "testées: —";
    }
    if ($("infoScanMode")) {
      const modeLabel = [];
      if (bruteOn) modeLabel.push("brute");
      if (dictOn) modeLabel.push("dict");
      $("infoScanMode").textContent = modeLabel.length ? `mode: ${modeLabel.join(" + ")}` : "mode: —";
    }
    // GPU/CPU detail line in the scan section
    if ($("scanRateDetail")) {
      const detailParts = [...gpuParts];
      if (cpuTotal > 0) detailParts.push(`CPU: ${formatCompact(cpuTotal)}/s`);
      $("scanRateDetail").textContent = detailParts.length ? detailParts.join(" · ") : "—";
    }

    if (synced) setPill("pillSync", "Chaîne OK", "ok", "Chaîne à jour");
    else if (running) setPill("pillSync", "Chaîne sync", "warn", s?.simple_status || "Sync en cours");
    else setPill("pillSync", "Chaîne off", "warn", "Core arrêté");

    if (indexOk) {
      const n = health.index_scripts || 0;
      setPill(
        "pillReady",
        `Idx ${formatCompact(n)}`,
        "ok",
        `Index chargé · ${formatNumber(n)} scripts`
      );
    } else {
      setPill("pillReady", "Idx …", "warn", "Index en chargement");
    }

    const msg = s?.simple_status || s?.message || "Tester des mots = onglet 1";
    setText("statusBarMsg", msg);
    if ($("statusBarMsg")) $("statusBarMsg").title = msg;

    updateTipWarning(window.__lastSnap, s, window.__lastHealth);
  }

  /** UTXO valid for tests if within this many hours of tip (user rule). */
  const UTXO_VALID_HOURS = 24;

  /**
   * Estimate how many hours the UTXO index lags behind "tip".
   * Prefer block timestamps; fallback = block lag × 10 min.
   */
  function estimateUtxoLagHours(snap, btc) {
    const idxH =
      snap?.block_height != null
        ? Number(snap.block_height)
        : (() => {
            const m = String(snap?.path || "").match(/(\d{5,7})/);
            return m ? Number(m[1]) : null;
          })();
    const coreBlocks = btc?.blocks != null ? Number(btc.blocks) : null;
    const coreHeaders = btc?.headers != null ? Number(btc.headers) : null;
    const tipH = Math.max(coreHeaders || 0, coreBlocks || 0);

    // 1) Time of UTXO tip block vs now (or Core tip time if available)
    let idxUnix = null;
    if (snap?.block_time_unix != null) idxUnix = Number(snap.block_time_unix);
    else if (snap?.block_time_utc) {
      const d = Date.parse(snap.block_time_utc);
      if (!Number.isNaN(d)) idxUnix = d / 1000;
    }

    let tipUnix = Date.now() / 1000;
    // If Core has a mediantime and is not stuck at genesis, use it as tip clock
    if (btc?.mediantime && Number(btc.mediantime) > 1_400_000_000) {
      tipUnix = Number(btc.mediantime);
    } else if (btc?.block_time && Number(btc.block_time) > 1_400_000_000) {
      tipUnix = Number(btc.block_time);
    }

    let hoursByTime = null;
    if (idxUnix != null && idxUnix > 0) {
      hoursByTime = Math.max(0, (tipUnix - idxUnix) / 3600);
    }

    // 2) Block lag → hours (≈ 10 min/block)
    let hoursByBlocks = null;
    if (idxH != null && tipH > idxH) {
      hoursByBlocks = ((tipH - idxH) * 10) / 60;
    } else if (idxH != null && tipH > 0 && tipH <= idxH) {
      hoursByBlocks = 0;
    }

    // Prefer the more conservative (larger) lag when both exist
    let hours = null;
    if (hoursByTime != null && hoursByBlocks != null) {
      hours = Math.max(hoursByTime, hoursByBlocks);
    } else {
      hours = hoursByTime != null ? hoursByTime : hoursByBlocks;
    }

    // 3) Last resort: snapshot file age (if built near tip when created)
    if (hours == null && snap?.age_hours != null) {
      hours = Number(snap.age_hours);
    }

    return {
      hours: hours != null && Number.isFinite(hours) ? hours : null,
      idxH,
      tipH: tipH || null,
      blockLag: idxH != null && tipH > 0 ? Math.max(0, tipH - idxH) : null,
      hoursByTime,
      hoursByBlocks,
    };
  }

  function updateTipWarning(snap, btc, health) {
    const warn = $("spendWarn");
    if (!warn) return;
    snap = snap || window.__lastSnap || health?.snapshot;
    btc = btc || health?.bitcoind;
    const alwaysOn = health?.core_utxo || window.__coreUtxo || null;
    if (alwaysOn) window.__coreUtxo = alwaysOn;
    const lag = estimateUtxoLagHours(snap, btc);
    // Prefer always-on pipeline lag if present
    if (alwaysOn?.utxo_lag_hours != null && Number.isFinite(Number(alwaysOn.utxo_lag_hours))) {
      lag.hours = Number(alwaysOn.utxo_lag_hours);
    }
    if (alwaysOn?.utxo_height != null) lag.idxH = Number(alwaysOn.utxo_height);
    if (alwaysOn?.headers != null) lag.tipH = Number(alwaysOn.headers);
    const coreBlocks = btc?.blocks != null ? Number(btc.blocks) : null;
    const coreHeaders = btc?.headers != null ? Number(btc.headers) : null;
    const coreSynced = !!btc?.is_synced;

    // Valid for tests: < 24h behind tip (user rule)
    const validForTests =
      lag.hours != null && lag.hours <= UTXO_VALID_HOURS;
    // Exact tip alignment (bonus / green wording)
    const atExactTip =
      coreSynced &&
      lag.idxH != null &&
      coreBlocks != null &&
      Math.abs(coreBlocks - lag.idxH) <= 6;

    window.__utxoValidForTests = validForTests;
    window.__utxoLagHours = lag.hours;

    const meta = $("tipWarnMeta");
    if (meta) {
      const parts = [];
      // Hauteurs exactes, jamais en K/M
      if (lag.idxH != null && (coreBlocks != null || lag.tipH != null)) {
        const tip = coreBlocks != null ? coreBlocks : lag.tipH;
        const lagB =
          tip != null && lag.idxH != null ? Math.max(0, tip - lag.idxH) : null;
        parts.push(
          `index UTXO bloc ${formatHeight(lag.idxH)} / Core tip ${formatHeight(tip)}` +
            (lagB != null ? ` (retard ${formatHeight(lagB)} blocs${formatLagTime(lagB)})` : "")
        );
      } else if (lag.idxH != null) {
        parts.push(
          `index UTXO: bloc ${formatHeight(lag.idxH)}${
            snap?.block_time_utc ? " · " + snap.block_time_utc : ""
          }`
        );
      } else {
        parts.push("index UTXO: hauteur inconnue");
      }
      if (coreBlocks != null) {
        parts.push(
          `Core: ${formatHeight(coreBlocks)} blocs` +
            (coreHeaders != null ? ` / ${formatHeight(coreHeaders)} headers` : "") +
            (btc?.initialblockdownload ? " · reindex/IBD" : "")
        );
      } else {
        parts.push("Core: ?");
      }
      if (lag.hours != null) {
        const hLabel =
          lag.hours < 1
            ? `${Math.round(lag.hours * 60)} min`
            : `${lag.hours.toFixed(1)} h`;
        parts.push(
          validForTests
            ? `retard ≈ ${hLabel} (< ${UTXO_VALID_HOURS}h → tests OK)`
            : `retard ≈ ${hLabel} (> ${UTXO_VALID_HOURS}h → tests non fiables)`
        );
      }
      if (lag.blockLag != null && lag.blockLag > 0) {
        parts.push(`${formatHeight(lag.blockLag)} blocs derrière le tip${formatLagTime(lag.blockLag)}`);
      }
      meta.textContent = parts.join("  ·  ");
    }

    const body = $("tipWarnBody");
    const strong = warn.querySelector("strong");
    warn.classList.remove("hidden");

    if (validForTests || atExactTip) {
      warn.classList.add("is-tip");
      warn.classList.remove("tip-stale");
      if (strong) {
        strong.textContent = atExactTip
          ? "✓ UTXO au tip — soldes fiables pour les tests"
          : `✓ UTXO valable pour les tests (< ${UTXO_VALID_HOURS} h du tip)`;
      }
      if (body) {
        const hLabel =
          lag.hours != null
            ? lag.hours < 1
              ? `${Math.round(lag.hours * 60)} min`
              : `${lag.hours.toFixed(1)} h`
            : "—";
        const idxL = lag.idxH != null ? formatHeight(lag.idxH) : "—";
        const tipL =
          coreBlocks != null
            ? formatHeight(coreBlocks)
            : lag.tipH != null
              ? formatHeight(lag.tipH)
              : "—";
        body.innerHTML =
          `Index UTXO au bloc <strong>${idxL}</strong> · tip Core <strong>${tipL}</strong>. ` +
          `L’index est à <strong>moins de ${UTXO_VALID_HOURS} h</strong> du tip estimé ` +
          `(retard ≈ <strong>${hLabel}</strong>). ` +
          `Tu peux t’en servir pour tester des clés / brainwallets. ` +
          `Vérifie quand même on-chain avant toute dépense.`;
      }
      // UTXO pill: green when valid
      if ($("pillUtxo") && lag.idxH != null) {
        const tipN = coreBlocks != null ? coreBlocks : lag.tipH;
        setPill(
          "pillUtxo",
          tipN != null
            ? `UTXO ${formatHeight(lag.idxH)} / ${formatHeight(tipN)}`
            : `UTXO ${formatHeight(lag.idxH)}`,
          "ok",
          `Index bloc ${formatHeight(lag.idxH)} · Core ${formatHeight(tipN)} · valable tests (<${UTXO_VALID_HOURS}h)`
        );
      }
    } else {
      warn.classList.remove("is-tip");
      warn.classList.add("tip-stale");
      const coreBlocksStale = alwaysOn?.blocks ?? btc?.blocks;
      const coreHeadersStale = alwaysOn?.headers ?? btc?.headers;
      const coreIbd = alwaysOn?.initialblockdownload ?? btc?.initialblockdownload;
      const coreRunning = alwaysOn?.core_running ?? btc?.running ?? btc?.process_running;
      const pct =
        alwaysOn?.verification_pct != null
          ? Number(alwaysOn.verification_pct)
          : btc?.verification_progress != null
            ? Number(btc.verification_progress)
            : null;

      if (strong) {
        if (!coreRunning) {
          strong.textContent = "⚠ Bitcoin Core arrêté — redémarrage automatique…";
        } else if (
          coreIbd ||
          (coreBlocksStale != null &&
            coreHeadersStale != null &&
            coreBlocksStale < coreHeadersStale - 3)
        ) {
          strong.textContent = "⏳ Core synchronise la chaîne — UTXO tip en attente (normal)";
        } else {
          const idxS = lag.idxH != null ? formatHeight(lag.idxH) : "?";
          const tipS =
            coreBlocksStale != null
              ? formatHeight(coreBlocksStale)
              : lag.tipH != null
                ? formatHeight(lag.tipH)
                : "?";
          strong.textContent = `⚠ UTXO bloc ${idxS} / Core ${tipS} — trop vieux pour des soldes fiables`;
        }
      }
      if (body) {
        const hLabel =
          lag.hours != null ? `${lag.hours.toFixed(1)} h` : "inconnu";
        const idxFull = lag.idxH != null ? formatHeight(lag.idxH) : "—";
        const tipFull =
          coreBlocksStale != null
            ? formatHeight(coreBlocksStale)
            : lag.tipH != null
              ? formatHeight(lag.tipH)
              : "—";
        const lagFull =
          lag.idxH != null &&
          (coreBlocksStale != null || lag.tipH != null)
            ? formatHeight(
                Math.max(
                  0,
                  Number(coreBlocksStale != null ? coreBlocksStale : lag.tipH) -
                    Number(lag.idxH)
                )
              )
            : "—";
        let html = "";
        if (!coreRunning) {
          html =
            "Le watchdog <code>Keep-Core-And-Utxo</code> doit relancer bitcoind sous peu. " +
            "S’il reste arrêté : lance <code>Install-AlwaysOn.bat</code> ou <code>Launch-BitcoinCore.ps1</code>.";
        } else if (
          coreIbd ||
          (coreBlocksStale != null &&
            coreHeadersStale != null &&
            coreBlocksStale < coreHeadersStale - 3)
        ) {
          html =
            `<strong>Bitcoin Core fonctionne</strong> et rattrape le tip : ` +
            `<strong>${formatHeight(coreBlocksStale)}</strong> / <strong>${formatHeight(coreHeadersStale)}</strong> blocs` +
            (pct != null ? ` · ~${pct.toFixed(2)} %` : "") +
            `. ` +
            `Index UTXO actuel : bloc <strong>${idxFull}</strong>. ` +
            `L’index UTXO offline ne peut être régénéré au tip <em>que</em> quand Core a fini (IBD = non). ` +
            `Auto-refresh : tâche <code>BTCSolver-Core-Utxo</code> (dumptxoutset dès le tip). ` +
            `<br>En attendant : tests = <em>candidats seulement</em> (retard UTXO ≈ <strong>${hLabel}</strong>). ` +
            `<strong>Ne coupe pas bitcoind.</strong>`;
        } else {
          html =
            `Index UTXO au bloc <strong>${idxFull}</strong> · tip Bitcoin Core <strong>${tipFull}</strong>` +
            ` · retard <strong>${lagFull}</strong> blocs (≈ <strong>${hLabel}</strong>).<br>` +
            `Règle : index UTXO <strong>valable pour les tests</strong> seulement s’il a ` +
            `<strong>&lt; ${UTXO_VALID_HOURS} h</strong> de retard sur le tip. ` +
            `Actuellement retard ≈ <strong>${hLabel}</strong>. ` +
            `Les hits restent des <em>candidats</em>. ` +
            `<br>Refresh auto au tip via <code>Keep-Core-And-Utxo.ps1</code>.`;
        }
        body.innerHTML = html;
      }
    }
  }

  function updateBtc(s) {
    if (!s) return;
    const running = !!(s.running || s.process_running);
    const rpc = !!s.rpc_ok;
    if ($("btcBadge")) {
      $("btcBadge").className = "status-badge " + (running ? "running" : "stopped");
      setText(
        "btcBadgeText",
        running && rpc ? "En ligne" : running ? "Process" : "Arrêté"
      );
    }
    // Hauteurs Core en entier exact (pas de K)
    setText("btcBlocks", s.blocks != null ? formatHeight(s.blocks) : "—");
    setText("btcHeaders", s.headers != null ? formatHeight(s.headers) : "—");
    if (s.blocks != null) window.__lastBtc = s;
    setText("btcPeers", s.connections != null ? s.connections : "—");
    setText(
      "btcIbd",
      s.initialblockdownload == null ? "—" : s.initialblockdownload ? "oui" : "non"
    );
    setText("btcMsgLine", s.simple_status || s.message || "—");
    setText("btcDatadir", s.datadir || "W:\\Bitcoin");
    const pct = s.verification_progress ?? s.sync_percentage ?? 0;
    if ($("btcProgress")) $("btcProgress").style.width = Math.min(100, Number(pct) || 0) + "%";
    setText("btcProgressLabel", (Number(pct) || 0).toFixed(4) + "%");

    if ($("btnBtcStart")) $("btnBtcStart").disabled = !!running && !!s.can_start === false;
    if ($("btnBtcStop")) $("btnBtcStop").disabled = !running;

    // Rafraîchir le bandeau UTXO « index / tip Core » dès qu’on a le tip
    if (window.__lastSnap) {
      updateUtxoDisplay(window.__lastSnap, {
        ...(window.__lastHealth || {}),
        bitcoind: s,
      });
    }

    updateStatusBar(s, window.__lastHealth);
  }

  const ALL_TRANSFORMS = [
    { id: "identity", label: "identity" },
    { id: "reverse_bytes", label: "reverse_bytes" },
    { id: "reverse_bits", label: "reverse_bits" },
    { id: "rotl8", label: "rotl8" },
    { id: "rotr8", label: "rotr8" },
    { id: "sha256", label: "sha256" },
    { id: "double_sha256", label: "double_sha256" },
  ];

  function shortenHex(h, head = 10, tail = 8) {
    if (!h || typeof h !== "string") return "—";
    const s = h.replace(/^0x/i, "").toLowerCase();
    if (s.length <= head + tail + 3) return s;
    return s.slice(0, head) + "…" + s.slice(-tail);
  }

  function selectedTransforms() {
    const map = [
      ["tfIdentity", "identity"],
      ["tfRevBytes", "reverse_bytes"],
      ["tfRevBits", "reverse_bits"],
      ["tfRotl8", "rotl8"],
      ["tfRotr8", "rotr8"],
      ["tfSha256", "sha256"],
      ["tfDouble", "double_sha256"],
    ];
    const out = [];
    for (const [id, name] of map) {
      if ($(id)?.checked) out.push(name);
    }
    return out.length ? out : ["identity"];
  }

  function renderActiveTransforms(list) {
    const box = $("activeTransforms");
    if (!box) return;
    const active = new Set((list || []).map((x) => String(x).toLowerCase()));
    if (!active.size) active.add("identity");
    box.innerHTML = ALL_TRANSFORMS.map((t) => {
      const on = active.has(t.id);
      return `<span class="transform-tag${on ? "" : " off"}">${on ? "✓" : "·"} ${t.label}</span>`;
    }).join("");
  }

  function syncTransformCheckboxes(list) {
    const active = new Set((list || []).map((x) => String(x).toLowerCase()));
    const map = {
      identity: "tfIdentity",
      reverse_bytes: "tfRevBytes",
      reverse_bits: "tfRevBits",
      rotl8: "tfRotl8",
      rotr8: "tfRotr8",
      sha256: "tfSha256",
      double_sha256: "tfDouble",
    };
    // Only auto-sync when scan is running (reflect live config), not wipe user edits
    if (!active.size) return;
    for (const [name, id] of Object.entries(map)) {
      if ($(id)) $(id).checked = active.has(name);
    }
  }

  function formatUtcLabel(s) {
    if (!s) return "—";
    // Accept ISO or already formatted
    try {
      const d = new Date(s);
      if (!Number.isNaN(d.getTime())) {
        return d.toISOString().replace("T", " ").replace(/\.\d{3}Z$/, " UTC");
      }
    } catch (_) {}
    return String(s);
  }

  let _lastArchHits = 0;
  let _lastHitFlashAt = 0;

  /** Flash écran + tuile + pill quand un hit apparaît ou augmente */
  function flashHitScreen(count, isFirst) {
    const now = Date.now();
    // Évite le spam si health poll toutes les 2–3 s
    if (now - _lastHitFlashAt < 2500) return;
    _lastHitFlashAt = now;

    playHitBeep();

    const overlay = $("hitFlashOverlay");
    const banner = $("hitFlashBanner");
    const text = $("hitFlashText");
    if (text) {
      text.textContent = isFirst
        ? `HIT — ${count} clé trouvée !`
        : `NOUVEAU HIT — ${count} clé(s) au total`;
    }
    if (overlay) {
      overlay.classList.remove("is-on");
      // reflow pour rejouer l’anim
      void overlay.offsetWidth;
      overlay.classList.add("is-on");
      setTimeout(() => overlay.classList.remove("is-on"), 1400);
    }
    if (banner) {
      banner.classList.add("is-on");
      setTimeout(() => banner.classList.remove("is-on"), 4200);
    }

    const tile = document.querySelector(".hit-tile");
    if (tile) {
      tile.classList.remove("hit-just-now");
      void tile.offsetWidth;
      tile.classList.add("hit-just-now");
      setTimeout(() => tile.classList.remove("hit-just-now"), 1200);
    }

    const pill = $("pillHits");
    if (pill) {
      pill.classList.add("pill-hit-flash");
      setTimeout(() => pill.classList.remove("pill-hit-flash"), 2500);
    }

    toast(
      isFirst
        ? `⚡ HIT ! ${count} clé avec activité/solde`
        : `⚡ Nouveau hit — total ${count}`,
      "success"
    );

    // Titre onglet clignotant (si onglet en arrière-plan)
    try {
      const base = document.title.replace(/^\(\d+\)\s*/, "").replace(/^⚡\s*/, "");
      document.title = `⚡ (${count}) ${base}`;
      let n = 0;
      const t = setInterval(() => {
        n += 1;
        document.title = n % 2 === 0 ? `⚡ HIT ${count}` : base;
        if (n >= 8) {
          clearInterval(t);
          document.title = count > 0 ? `(${count}) ${base}` : base;
        }
      }, 400);
    } catch (_) {}
  }

  function updateHitsDisplay(n, archive) {
    const hits = Number(n) || 0;
    const archCount = archive?.count != null ? Number(archive.count) : hits;
    // Priorité : clés avec solde ; sinon count archive (activité)
    const withBalNow =
      archive?.with_balance != null
        ? Number(archive.with_balance)
        : archive?.count != null
          ? Number(archive.count)
          : hits;
    const signal = Math.max(withBalNow, archCount, hits);

    // Flash si on passe de 0 → >0 ou si le compteur augmente
    if (signal > _lastArchHits) {
      const isFirst = _lastArchHits <= 0 && signal > 0;
      if (signal > 0) flashHitScreen(signal, isFirst);
    }
    if (signal > 0) _lastArchHits = Math.max(_lastArchHits, signal);

    const withBal =
      archive?.with_balance != null ? Number(archive.with_balance) : hits;
    const actOnly =
      archive?.activity_only != null ? Number(archive.activity_only) : 0;
    setText("matchesFound", formatNumber(archCount));
    setText("infoHitsBig", formatNumber(archCount));
    if ($("infoHitsSub")) {
      $("infoHitsSub").textContent =
        archCount > 0
          ? `${withBal} solde · ${actOnly} activité sans solde · archive data/keys-archive.json`
          : "aucune clé avec activité on-chain pour l’instant";
    }
    const tile = document.querySelector(".hit-tile");
    if (tile) tile.classList.toggle("has-hits", archCount > 0);
    setPill(
      "pillHits",
      `Hits ${formatCompact(archCount)}`,
      archCount > 0 ? "ok" : "",
      archCount > 0
        ? `${formatNumber(archCount)} clé(s) actives (solde ou historique) — data/keys-archive.json`
        : "Aucune clé avec activité on-chain"
    );
  }

  /**
   * Badges Core / UTXO dans la tuile index :
   * - Core : vert si bitcoind roule + tip frais; jaune si > 1 h de retard; rouge si arrêté
   * - UTXO : vert si retard < 24 h ; jaune si ≥ 24 h mais rebuild en cours ;
   *          rouge si ≥ 24 h et PAS de rebuild
   */
  function applyUtxoStatusBadges({
    coreRunning,
    lagHours,
    blockLag,
    idxH,
    coreTip,
    rebuildInProgress,
    coreTipAgeSec,
  }) {
    const badgeCore = $("badgeCore");
    const badgeUtxo = $("badgeUtxo");
    const tile = $("utxoStatusTile");
    const setBadge = (el, cls, text, title) => {
      if (!el) return;
      el.className = "status-badge " + cls;
      el.textContent = text;
      el.title = title || text;
    };

    // --- Core ---
    if (coreRunning === true) {
      const coreStale = coreTipAgeSec != null && coreTipAgeSec > 3600;
      if (coreStale) {
        const lagH = (coreTipAgeSec / 3600).toFixed(1);
        setBadge(
          badgeCore,
          "is-yellow",
          coreTip != null ? `Core lent · ${formatHeight(coreTip)}` : "Core lent",
          `Tip Core il y a ${lagH} h (dernier bloc > 1 h)`
        );
      } else {
        setBadge(
          badgeCore,
          "is-green",
          coreTip != null ? `Core ON · ${formatHeight(coreTip)}` : "Core ON",
          coreTip != null
            ? `Bitcoin Core en marche · tip bloc ${coreTip}`
            : "Bitcoin Core en marche"
        );
      }
    } else if (coreRunning === false) {
      setBadge(
        badgeCore,
        "is-red",
        "Core OFF",
        "Bitcoin Core arrêté — soldes/tip non fiables"
      );
    } else {
      setBadge(badgeCore, "is-muted", "Core · ?", "État Core inconnu");
    }

    // --- UTXO ---
    // Priorité : rebuild en cours = jaune (même si très en retard)
    let utxoCls = "is-muted";
    let utxoTxt = "UTXO · ?";
    let utxoTitle = "Fraîcheur UTXO inconnue";
    if (rebuildInProgress) {
      utxoCls = "is-yellow";
      utxoTxt =
        idxH != null
          ? `UTXO rebuild… · ${formatHeight(idxH)}`
          : "UTXO rebuild…";
      utxoTitle =
        "Recréation de l’index en cours (dumptxoutset / dump_to_flat) — pas encore rouge";
    } else if (lagHours != null && Number.isFinite(lagHours)) {
      if (lagHours < UTXO_VALID_HOURS) {
        utxoCls = "is-green";
        utxoTxt =
          idxH != null
            ? `UTXO OK · ${formatHeight(idxH)}`
            : "UTXO OK (< 24 h)";
        utxoTitle = `Index à jour (< ${UTXO_VALID_HOURS} h du tip) · retard ≈ ${lagHours.toFixed(1)} h`;
      } else {
        // ≥ 24 h et pas de rebuild → rouge
        utxoCls = "is-red";
        utxoTxt =
          idxH != null
            ? `UTXO vieux · ${formatHeight(idxH)}`
            : "UTXO vieux (≥ 24 h)";
        utxoTitle =
          `Retard ≈ ${lagHours.toFixed(1)} h (≥ ${UTXO_VALID_HOURS} h) et aucune recréation en cours` +
          (blockLag != null ? ` · ${formatHeight(blockLag)} blocs${formatLagTime(blockLag)}` : "");
      }
    } else if (blockLag != null) {
      // fallback sans heures : ~144 blocs ≈ 24 h
      if (blockLag < 144) {
        utxoCls = "is-green";
        utxoTxt =
          idxH != null
            ? `UTXO OK · ${formatHeight(idxH)}`
            : "UTXO OK";
        utxoTitle = `Retard ${formatHeight(blockLag)} blocs${formatLagTime(blockLag)} (< ~24 h)`;
      } else {
        utxoCls = "is-red";
        utxoTxt =
          idxH != null
            ? `UTXO vieux · ${formatHeight(idxH)}`
            : "UTXO vieux";
        utxoTitle = `Retard ${formatHeight(blockLag)} blocs${formatLagTime(blockLag)} (≥ ~24 h), pas de rebuild`;
      }
    }

    setBadge(badgeUtxo, utxoCls, utxoTxt, utxoTitle);

    // Teinte tuile
    if (tile) {
      tile.classList.remove("state-ok", "state-warn", "state-bad", "state-core-down");
      if (coreRunning === false) {
        tile.classList.add("state-core-down");
      } else if (utxoCls === "is-green") {
        tile.classList.add("state-ok");
      } else if (utxoCls === "is-yellow") {
        tile.classList.add("state-warn");
      } else if (utxoCls === "is-red") {
        tile.classList.add("state-bad");
      }
    }
  }

  function isUtxoRebuildInProgress(health, alwaysOn) {
    if (health?.utxo_rebuild_in_progress) return true;
    if (window.__utxoRebuildInProgress) return true;
    const ao = alwaysOn || health?.core_utxo || window.__coreUtxo || null;
    if (ao?.utxo_rebuild_in_progress) return true;
    if (ao?.last_refresh?.reason === "in_progress") return true;
    return false;
  }

  function updateUtxoDisplay(snap, health) {
    if (!snap && health?.snapshot) snap = health.snapshot;
    // Même sans snapshot, mettre à jour les pastilles Core / rebuild
    if (!snap) {
      const btc = health?.bitcoind || window.__lastBtc || null;
      const alwaysOn = health?.core_utxo || window.__coreUtxo || null;
      const coreRunning =
        btc?.running != null
          ? !!btc.running
          : alwaysOn?.core_running != null
            ? !!alwaysOn.core_running
            : null;
      const coreTip =
        btc?.blocks != null
          ? Number(btc.blocks)
          : alwaysOn?.blocks != null
            ? Number(alwaysOn.blocks)
            : null;
      const coreTipAgeSec1 = btc?.block_time
        ? (Date.now() / 1000 - Number(btc.block_time))
        : null;
      applyUtxoStatusBadges({
        coreRunning,
        lagHours:
          alwaysOn?.utxo_lag_hours != null
            ? Number(alwaysOn.utxo_lag_hours)
            : null,
        blockLag: null,
        idxH: alwaysOn?.utxo_height != null ? Number(alwaysOn.utxo_height) : null,
        coreTip,
        rebuildInProgress: isUtxoRebuildInProgress(health, alwaysOn),
        coreTipAgeSec: coreTipAgeSec1,
      });
      return;
    }

    const height =
      snap.block_height != null
        ? Number(snap.block_height)
        : (() => {
            const m = String(snap.path || "").match(/(\d{5,7})/);
            return m ? Number(m[1]) : null;
          })();
    // Tip Core : health.bitcoind / core_utxo / cache global
    const btc = health?.bitcoind || window.__lastBtc || null;
    const alwaysOn = health?.core_utxo || window.__coreUtxo || null;
    const coreRunning =
      btc?.running != null
        ? !!btc.running
        : btc?.process_running != null
          ? !!btc.process_running
          : alwaysOn?.core_running != null
            ? !!alwaysOn.core_running
            : null;
    const coreTip =
      btc?.blocks != null
        ? Number(btc.blocks)
        : alwaysOn?.blocks != null
          ? Number(alwaysOn.blocks)
          : alwaysOn?.headers != null
            ? Number(alwaysOn.headers)
            : null;
    const coreHeaders =
      btc?.headers != null
        ? Number(btc.headers)
        : alwaysOn?.headers != null
          ? Number(alwaysOn.headers)
          : null;
    const blockLag =
      height != null && coreTip != null ? Math.max(0, coreTip - height) : null;

    // Retard en heures (pipeline always-on ou estimation)
    let lagHours =
      alwaysOn?.utxo_lag_hours != null && Number.isFinite(Number(alwaysOn.utxo_lag_hours))
        ? Number(alwaysOn.utxo_lag_hours)
        : null;
    if (lagHours == null) {
      const lagEst = estimateUtxoLagHours(snap, btc);
      lagHours = lagEst.hours;
    }
    const rebuildInProgress = isUtxoRebuildInProgress(health, alwaysOn);
    const coreTipAgeSec2 = btc?.block_time
      ? (Date.now() / 1000 - Number(btc.block_time))
      : null;

    applyUtxoStatusBadges({
      coreRunning,
      lagHours,
      blockLag,
      idxH: height,
      coreTip,
      rebuildInProgress,
      coreTipAgeSec: coreTipAgeSec2,
    });

    let blockDate =
      snap.block_time_utc ||
      (snap.block_time_unix
        ? formatUtcLabel(new Date(Number(snap.block_time_unix) * 1000).toISOString())
        : null);
    const built =
      snap.built_at ||
      snap.file_modified_utc ||
      (snap.age_hours != null ? `fichier ~${Number(snap.age_hours).toFixed(1)} h` : null);
    const hash = snap.base_block_hash || null;
    const scripts = snap.num_scripts ?? health?.index_scripts ?? snap.index_scripts;
    const utxos = snap.num_utxos;

    // Bandeau principal : hauteurs exactes (pas de "935.0 K")
    if (height != null && coreTip != null) {
      setText(
        "infoUtxoBlock",
        `${formatHeight(height)} / ${formatHeight(coreTip)}`
      );
      if ($("infoUtxoBlock")) {
        $("infoUtxoBlock").title =
          `Index UTXO = bloc ${height} · Bitcoin Core tip = bloc ${coreTip}` +
          (blockLag != null ? ` · retard ${formatHeight(blockLag)} blocs${formatLagTime(blockLag)}` : "");
      }
      setText(
        "infoUtxoVsCore",
        `index ${formatHeight(height)} · Core tip ${formatHeight(coreTip)}` +
          (blockLag != null && blockLag > 0
            ? ` · retard ${formatHeight(blockLag)} blocs${formatLagTime(blockLag)}`
            : blockLag === 0
              ? " · au tip"
              : "")
      );
    } else if (height != null) {
      setText("infoUtxoBlock", `bloc ${formatHeight(height)}`);
      if ($("infoUtxoBlock")) {
        $("infoUtxoBlock").title = `Index UTXO = bloc ${height} (tip Core inconnu)`;
      }
      setText("infoUtxoVsCore", "Core tip: (inconnu — Core offline ?)");
    } else {
      setText("infoUtxoBlock", "bloc —");
      setText("infoUtxoVsCore", "Core tip: —");
    }

    setText("infoUtxoDate", blockDate ? `date du bloc: ${blockDate}` : "date du bloc: (indisponible — Core pas au tip / reindex)");
    setText("infoUtxoBuilt", built ? `index généré: ${formatUtcLabel(built)}` : "index généré: —");
    setText("infoUtxoHash", hash ? `hash: ${hash}` : "hash: —");
    if ($("infoUtxoHash") && hash) $("infoUtxoHash").title = hash;

    // Panneau détail (nombres exacts)
    setText("utxoBlockHeight", height != null ? formatHeight(height) : "—");
    if ($("utxoBlockHeight") && height != null) {
      $("utxoBlockHeight").title = String(height);
    }
    setText(
      "utxoCoreTip",
      coreTip != null
        ? formatHeight(coreTip) +
            (coreHeaders != null && coreHeaders !== coreTip
              ? ` (headers ${formatHeight(coreHeaders)})`
              : "")
        : "—"
    );
    if ($("utxoCoreTip") && coreTip != null) {
      $("utxoCoreTip").title = String(coreTip);
    }
    setText(
      "utxoBlockLag",
      blockLag != null
        ? blockLag === 0
          ? "0 (au tip)"
          : formatHeight(blockLag)
        : "—"
    );
    setText("utxoBlockDate", blockDate || "—");
    setText("utxoBuiltAt", built ? formatUtcLabel(built) : "—");
    setText("utxoBlockHash", hash || "—");
    if ($("utxoBlockHash") && hash) $("utxoBlockHash").title = hash;
    const counts =
      scripts != null || utxos != null
        ? `${scripts != null ? formatNumber(scripts) + " scripts" : "— scripts"} · ${
            utxos != null ? formatNumber(utxos) + " UTXOs" : "— UTXOs"
          }`
        : "—";
    setText("utxoCounts", counts);

    const ageH = snap.age_hours;
    // Pill : index / tip en clair
    const pillTxt =
      height != null && coreTip != null
        ? `UTXO ${formatHeight(height)} / ${formatHeight(coreTip)}`
        : height != null
          ? `UTXO ${formatHeight(height)}`
          : "UTXO …";
    const tip = [
      height != null ? `index bloc ${height}` : null,
      coreTip != null ? `Core tip ${coreTip}` : null,
      blockLag != null ? `retard ${formatHeight(blockLag)} blocs${formatLagTime(blockLag)}` : null,
      snap.block_time_utc || null,
      ageH != null ? `âge fichier ${Number(ageH).toFixed(1)} h` : null,
      snap.path || null,
    ]
      .filter(Boolean)
      .join(" · ");
    setPill(
      "pillUtxo",
      pillTxt,
      snap.fresh === false || (blockLag != null && blockLag > 144) ? "warn" : "ok",
      tip || pillTxt
    );
  }

  function updateScan(s, health) {
    if (!s || s.error) return;
    const run = !!s.running;
    const arch = health?.keys_archive || window.__lastHealth?.keys_archive || null;
    const hits =
      arch?.count != null
        ? arch.count
        : s.keys_with_balance != null
          ? s.keys_with_balance
          : s.matches_found || 0;
    const live = run
      ? (s.keys_per_sec_live != null && s.keys_per_sec_live > 0
        ? s.keys_per_sec_live
        : s.keys_per_sec)
      : 0;
    const avg = run
      ? (s.keys_per_sec_avg != null ? s.keys_per_sec_avg : s.keys_per_sec)
      : 0;
    setText("keysPerSec", formatNumber(live));
    setText("keysTested", formatNumber(s.keys_tested));
    setText(
      "keysPerSecAvg",
      avg != null ? formatNumber(avg) : "—"
    );
    // Heure dernière MAJ — clignote / change chaque intervalle pour prouver le live
    const maj =
      s.stats_updated_at ||
      (s.timestamp
        ? String(s.timestamp).replace(/.*T/, "").replace(/\+.*/, "").slice(0, 8)
        : null);
    if ($("statsUpdatedAt")) {
      const el = $("statsUpdatedAt");
      const prev = el.dataset.lastMaj || "";
      el.textContent = maj || (run ? "…" : "—");
      el.title = s.timestamp || "";
      if (maj && maj !== prev) {
        el.style.color = "var(--ok, #3ddc97)";
        el.dataset.lastMaj = maj;
        setTimeout(() => {
          el.style.color = "";
        }, 600);
      } else if (!run) {
        el.style.color = "";
        el.dataset.lastMaj = "";
      }
    }
    // Détail par carte GPU + CPU
    if ($("scanRateDetail")) {
      const parts = [];
      if (Array.isArray(s.gpu_rates) && s.gpu_rates.length) {
        for (const g of s.gpu_rates) {
          parts.push(
            `GPU${g.id}: ${formatNumber(g.keys_per_sec || 0)}/s (${formatNumber(g.keys_tested || 0)})`
          );
        }
      }
      if (s.cpu_threads || s.cpu_keys_per_sec || s.cpu_keys_tested) {
        parts.push(
          `CPU×${s.cpu_threads || "?"} : ${formatNumber(s.cpu_keys_per_sec || 0)}/s (${formatNumber(s.cpu_keys_tested || 0)})`
        );
      }
      $("scanRateDetail").textContent = parts.length
        ? parts.join(" · ")
        : run
          ? "vitesses par carte en attente…"
          : "—";
    }
    updateHitsDisplay(hits, arch);
    setText("gpuUtil", s.gpu_util != null ? Number(s.gpu_util).toFixed(0) + "%" : "—");
    setText("currentPosition", s.current_position || s.range_end || "—");

    setText(
      "infoScanRate",
      live != null ? `${formatNumber(live)} /s` : "— /s"
    );
    setText(
      "infoScanTested",
      s.keys_tested != null ? `testées: ${formatNumber(s.keys_tested)}` : "testées: —"
    );

    // Hex COMPLET (pas de raccourci …) pour De / À / curseur
    const rs = s.range_start || s.start_key || null;
    const re = s.window_end || s.range_end || null;
    const rsShow = rs || "—";
    const reShow = re || "—";
    setText("rangeStart", rsShow);
    setText("rangeEnd", reShow);
    if ($("rangeStart")) {
      $("rangeStart").title = rs || "";
      $("rangeStart").dataset.full = rs || "";
    }
    if ($("rangeEnd")) {
      $("rangeEnd").title = re || "";
      $("rangeEnd").dataset.full = re || "";
    }
    // Champ éditable départ : ne pas écraser si l'utilisateur tape
    if ($("rangeStartEdit") && rs && document.activeElement !== $("rangeStartEdit")) {
      if (!$("rangeStartEdit").dataset.dirty) {
        $("rangeStartEdit").value = rs;
      }
    }
    if (s.range_step != null && $("rangeStep") && document.activeElement !== $("rangeStep")) {
      $("rangeStep").value = String(s.range_step);
      updateRangeStepHint(s.range_step);
    }
    if (s.ranges_done != null) {
      setText(
        "rangesDoneLabel",
        `${formatNumber(s.ranges_done)} plage(s) journalisée(s)`
      );
    }
    if ($("btnCopyRangeStart")) {
      $("btnCopyRangeStart").style.display = rs ? "inline-block" : "none";
      $("btnCopyRangeStart").onclick = () => {
        if (rs) copyTextToClipboard(rs).then(() => toast("De copié", "success"));
      };
    }
    if ($("btnCopyRangeEnd")) {
      $("btnCopyRangeEnd").style.display = re ? "inline-block" : "none";
      $("btnCopyRangeEnd").onclick = () => {
        if (re) copyTextToClipboard(re).then(() => toast("À copié", "success"));
      };
    }

    if ($("rangeSummary")) {
      $("rangeSummary").textContent =
        s.range_summary ||
        (run ? "Scan en cours…" : "Scan arrêté — se relance auto si GPU libre (pas de scan listes)");
    }

    const mode = (s.mode || "").toLowerCase();
    if ($("rangeStartLabel")) {
      $("rangeStartLabel").textContent =
        mode === "sequential"
          ? "De (départ fenêtre — hex complet)"
          : "De (min threads — hex complet)";
    }
    if ($("rangeEndLabel")) {
      $("rangeEndLabel").textContent =
        mode === "sequential"
          ? "À (fin fenêtre — hex complet)"
          : "À (max threads — hex complet)";
    }

    // Échantillon threads — hex complets
    if ($("threadKeysSample")) {
      if (Array.isArray(s.thread_keys) && s.thread_keys.length) {
        $("threadKeysSample").textContent = s.thread_keys
          .map((k, i) => `T${i}: ${k}`)
          .join("\n");
        $("threadKeysSample").title = s.thread_keys.join("\n");
      } else {
        $("threadKeysSample").textContent = run ? "…" : "—";
      }
    }

    if ($("scanModePill")) {
      if (!run) {
        setPill("scanModePill", "ARRÊTÉ (auto GPU idle)", "warn");
        setText(
          "rangeHint",
          "Pas de process brute — redémarrage auto ~15 s si pas de scan listes GPU"
        );
        setText("infoScanMode", "mode: off");
      } else if (mode === "random") {
        setPill("scanModePill", "RANDOM · live", "warn");
        setText(
          "rangeHint",
          "Random : De/À = min→max des dernières clés des threads (échantillons, pas une plage continue)"
        );
        setText("infoScanMode", "mode: RANDOM");
      } else if (mode === "sequential") {
        setPill("scanModePill", "SÉQUENTIEL · live", "ok");
        const step = s.range_step || 1073741824;
        setText(
          "rangeHint",
          rs && re
            ? `Séquentiel : fenêtre De → À (${formatNumber(step)} clés / pas). Curseur live plus bas. Journal = pas de retest.`
            : "Séquentiel : départ fenêtre → fin fenêtre (pas 2^30 par défaut)"
        );
        setText("infoScanMode", "mode: SÉQUENTIEL");
      } else {
        setPill("scanModePill", "actif", "ok");
        setText("infoScanMode", "mode: actif");
      }
    }

    if (Array.isArray(s.transforms) && s.transforms.length) {
      renderActiveTransforms(s.transforms);
      if (run) syncTransformCheckboxes(s.transforms);
    } else {
      renderActiveTransforms(selectedTransforms());
    }

    if ($("scanRandom") && mode) {
      // reflect mode only while running
      if (run) $("scanRandom").checked = mode === "random";
    }

    if ($("btnStart")) $("btnStart").disabled = run;
    if ($("btnStop")) $("btnStop").disabled = !run;

    // Status bar: fixed short label (range only in tooltip — avoids layout jump)
    if (run) {
      const rate = live;
      const rangeTip =
        rs || re
          ? `${rs || "?"} → ${re || "?"}`
          : s.current_position || "";
      const majTip = maj ? `MAJ ${maj}` : null;
      // GPU + CPU breakdown for tooltip
      const gpuParts = [];
      if (Array.isArray(s.gpu_rates)) {
        for (const g of s.gpu_rates) {
          gpuParts.push(`GPU${g.id}: ${formatCompact(g.keys_per_sec || g.keys_per_sec_avg || 0)}/s`);
        }
      }
      const cpuRate = Number(s.cpu_keys_per_sec || 0);
      const hwParts = [];
      if (gpuParts.length) hwParts.push(gpuParts.join(" + "));
      if (cpuRate > 0) hwParts.push(`CPU: ${formatCompact(cpuRate)}/s`);
      setPill(
        "pillScan",
        rate != null ? `Scan ${formatCompact(rate)}/s` : "Scan ON",
        "ok",
        [
          ...hwParts,
          rate != null ? `${formatNumber(rate)} keys/s live` : null,
          s.keys_tested != null ? `${formatNumber(s.keys_tested)} testées` : null,
          majTip,
          rangeTip || null,
        ]
          .filter(Boolean)
          .join(" · ")
      );
    } else {
      setPill("pillScan", "Scan off", "warn", "Scan de fond arrêté");
    }
  }

  async function refreshBtc() {
    try {
      updateBtc(await api("/api/bitcoind/status"));
    } catch (e) {
      updateBtc({ running: false, simple_status: "Core: inaccessible" });
    }
  }

  async function refreshSnap() {
    try {
      const s = await api("/api/snapshot/info");
      window.__lastSnap = s;
      setText("snapPath", s.path || "—");
      setText("snapAge", s.age_hours != null ? s.age_hours.toFixed(1) + " h" : "—");
      setText("snapSize", s.size_mb != null ? s.size_mb.toFixed(0) + " Mo" : "—");
      setText("indexLoaded", s.index_loaded ? "oui" : "non");
      setText(
        "indexScripts",
        s.num_scripts != null
          ? formatNumber(s.num_scripts)
          : s.index_scripts != null
            ? formatNumber(s.index_scripts)
            : "—"
      );
      updateUtxoDisplay(s, window.__lastHealth);
      updateTipWarning(s, window.__lastHealth?.bitcoind, window.__lastHealth);
    } catch (_) {}
  }

  async function refreshHealth() {
    try {
      const h = await api("/api/system/health", { timeoutMs: 25000 });
      window.__lastHealth = h;
      if (h.core_utxo) window.__coreUtxo = h.core_utxo;
      window.__utxoRebuildInProgress = !!h.utxo_rebuild_in_progress;
      setText(
        "footerHealth",
        `v${h.version || "?"} · index=${h.index_loaded ? "OK" : "off"} · status=${h.status || "?"}`
      );
      try {
        if ($("healthBox")) {
          // Compact dump — full peer/raw blobs removed server-side
          $("healthBox").textContent = JSON.stringify(h, null, 2);
        }
      } catch (_) {}
      try {
        if (h.bitcoind) updateBtc(h.bitcoind);
        else updateStatusBar(null, h);
      } catch (e) {
        console.warn("updateBtc", e);
      }
      try {
        if (h.scan) updateScan(h.scan, h);
        else if (h.keys_archive) updateHitsDisplay(h.keys_archive.count, h.keys_archive);
      } catch (e) {
        console.warn("updateScan", e);
      }
      try {
        if (h.snapshot) {
          window.__lastSnap = h.snapshot;
          updateUtxoDisplay(h.snapshot, h);
        } else if (window.__lastSnap) {
          updateUtxoDisplay(window.__lastSnap, h);
        }
        updateTipWarning(window.__lastSnap || h.snapshot, h.bitcoind, h);
      } catch (e) {
        console.warn("updateUtxo/tip", e);
      }
      // Success path always restores index pill if loaded
      if (h.index_loaded) {
        setPill(
          "pillReady",
          `Idx ${formatCompact(h.index_scripts || 0)}`,
          "ok",
          `Index chargé · ${formatNumber(h.index_scripts || 0)} scripts`
        );
      } else {
        setPill("pillReady", "Idx …", "warn", "Index en chargement");
      }
    } catch (e) {
      console.error("refreshHealth", e);
      setText("footerHealth", "API: " + (e.name === "AbortError" ? "timeout" : e.message || e));
      setPill("pillReady", "API …", "warn", e.message || "API indisponible");
    }
  }

  async function refreshScan() {
    try {
      updateScan(await api("/api/scan/stats"));
    } catch (_) {}
  }

  async function btcAction(path, label) {
    setMsg("btcMsg", label + "…");
    try {
      const r = await api(path, { method: "POST", body: "{}" });
      setMsg("btcMsg", r.message || "OK", "success");
      toast(r.message || label, "success");
      setTimeout(refreshBtc, 2000);
    } catch (e) {
      setMsg("btcMsg", e.message, "error");
      toast(e.message, "error");
    }
  }

  $("btnBtcRestart")?.addEventListener("click", () =>
    btcAction("/api/bitcoind/restart", "Relance")
  );
  $("btnBtcStart")?.addEventListener("click", () => btcAction("/api/bitcoind/start", "Start"));
  $("btnBtcStop")?.addEventListener("click", () => btcAction("/api/bitcoind/stop", "Stop"));
  $("btnBtcRefresh")?.addEventListener("click", refreshBtc);
  $("btnBarRestart")?.addEventListener("click", () =>
    btcAction("/api/bitcoind/restart", "Relance")
  );
  $("btnBarRefresh")?.addEventListener("click", () => {
    refreshBtc();
    refreshHealth();
    refreshSnap();
  });
  $("btnBarReloadIndex")?.addEventListener("click", async () => {
    try {
      const r = await api("/api/snapshot/reload", { method: "POST", body: "{}" });
      toast(
        r.success ? `Index: ${formatNumber(r.index_scripts)} scripts` : "Échec index",
        r.success ? "success" : "error"
      );
      refreshHealth();
      refreshSnap();
    } catch (e) {
      toast(e.message, "error");
    }
  });
  $("btnSnapReload")?.addEventListener("click", () => $("btnBarReloadIndex")?.click());
  $("btnSnapRefresh")?.addEventListener("click", async () => {
    if (!confirm("Rebuild UTXO (très long) ?")) return;
    setMsg("snapMsg", "Rebuild…");
    try {
      await api("/api/snapshot/refresh", { method: "POST", body: "{}" });
      setMsg("snapMsg", "OK — recharge l’index", "success");
    } catch (e) {
      setMsg("snapMsg", e.message, "error");
    }
  });

  // Brute — CPU % des cœurs (défaut 50), configurable
  function logicalCores() {
    return (
      window.__scanLogicalCores ||
      navigator.hardwareConcurrency ||
      24
    );
  }
  function readCpuPct() {
    const n = parseInt($("scanCpuPctNum")?.value ?? $("scanCpuPct")?.value ?? "50", 10);
    if (Number.isNaN(n)) return 50;
    return Math.max(0, Math.min(100, n));
  }
  function updateCpuPctUi(pct) {
    const p = Math.max(0, Math.min(100, Number(pct) || 0));
    if ($("scanCpuPct")) $("scanCpuPct").value = String(p);
    if ($("scanCpuPctNum")) $("scanCpuPctNum").value = String(p);
    if ($("scanCpuPctLabel")) $("scanCpuPctLabel").textContent = p + " %";
    const cores = logicalCores();
    const fixed = parseInt($("threads")?.value || "0", 10) || 0;
    let workers;
    if (fixed > 0) {
      workers = fixed;
    } else if (p === 0) {
      workers = 0;
    } else {
      workers = Math.max(1, Math.floor((cores * p) / 100));
    }
    if ($("scanCpuThreadsHint")) {
      $("scanCpuThreadsHint").textContent =
        fixed > 0
          ? `→ ${workers} workers forcés (threads fixes) · ${cores} cœurs détectés`
          : p === 0
            ? `→ 0 worker CPU (GPU only) · ${cores} cœurs`
            : `→ ${workers} workers CPU (${p}% de ${cores} cœurs) — actif par défaut`;
    }
  }
  function updateRangeStepHint(step) {
    const n = Number(step) || 0;
    if (!$("rangeStepHint")) return;
    if (n <= 0) {
      $("rangeStepHint").textContent = "—";
      return;
    }
    const exp = Math.log2(n);
    const expStr = Number.isInteger(exp) ? `2^${exp}` : `≈2^${exp.toFixed(2)}`;
    $("rangeStepHint").textContent = `= ${expStr} = ${formatNumber(n)} clés`;
  }

  async function refreshRangesLog() {
    try {
      const log = await api("/api/scan/ranges");
      const ranges = Array.isArray(log.ranges) ? log.ranges : [];
      setText(
        "rangesDoneLabel",
        `${formatNumber(ranges.length)} plage(s) journalisée(s)`
      );
      if ($("rangesLogList")) {
        if (!ranges.length) {
          $("rangesLogList").textContent =
            "Aucune plage terminée — le journal se remplit quand un À est atteint.";
        } else {
          // plus récentes en haut
          const lines = ranges
            .slice()
            .reverse()
            .slice(0, 40)
            .map((r, i) => {
              const n = ranges.length - i;
              return `#${n} De ${r.start}\n    À ${r.end}\n    ${formatNumber(r.keys || 0)} clés · ${r.completed_at || ""}`;
            });
          $("rangesLogList").textContent = lines.join("\n\n");
        }
      }
      if (log.range_step && $("rangeStep") && document.activeElement !== $("rangeStep")) {
        $("rangeStep").value = String(log.range_step);
        updateRangeStepHint(log.range_step);
      }
      if (log.manual_start && $("rangeStartEdit") && !$("rangeStartEdit").dataset.dirty) {
        $("rangeStartEdit").value = log.manual_start;
      }
      if (log.current?.start && $("rangeStartEdit") && !$("rangeStartEdit").dataset.dirty) {
        // préfère fenêtre courante pour l'édition
        $("rangeStartEdit").value = log.current.start;
      }
    } catch (_) {}
  }

  function buildScanConfigBody() {
    const transforms = selectedTransforms();
    const cpu_pct = readCpuPct();
    const threads = parseInt($("threads")?.value || "0", 10) || 0;
    const range_step =
      parseInt($("rangeStep")?.value || "1073741824", 10) || 1073741824;
    const startEdit = $("rangeStartEdit")?.value?.trim() || null;
    const startForm = $("startKey")?.value?.trim() || null;
    return {
      use_gpu: $("useGpu")?.checked ?? true,
      threads,
      cpu_pct,
      batch_size: parseInt($("batchSize")?.value || "4194304", 10) || 4194304,
      start_key: startEdit || startForm || null,
      count: 0,
      addr_types: "legacy,segwit,wrapped,taproot",
      stats_interval: 5,
      progress_interval: 15,
      random: $("scanRandom")?.checked ?? false,
      transforms,
      gpus: "0,1,2",
      range_step,
      use_range_log: true,
    };
  }
  async function persistScanConfig() {
    try {
      await api("/api/scan/config", {
        method: "POST",
        body: JSON.stringify(buildScanConfigBody()),
      });
    } catch (_) {}
  }
  async function loadScanConfig() {
    try {
      const c = await api("/api/scan/config");
      if (c.logical_cores) window.__scanLogicalCores = c.logical_cores;
      if (c.cpu_pct != null) updateCpuPctUi(c.cpu_pct);
      else updateCpuPctUi(50);
      if (c.threads != null && $("threads")) $("threads").value = String(c.threads);
      if (c.use_gpu != null && $("useGpu")) $("useGpu").checked = !!c.use_gpu;
      if (c.random != null && $("scanRandom")) $("scanRandom").checked = !!c.random;
      if (c.start_key && $("startKey") && !$("startKey").value)
        $("startKey").value = c.start_key;
      if (c.range_step && $("rangeStep")) {
        $("rangeStep").value = String(c.range_step);
        updateRangeStepHint(c.range_step);
      }
      if (Array.isArray(c.transforms) && c.transforms.length) {
        syncTransformCheckboxes(c.transforms);
        renderActiveTransforms(c.transforms);
      }
      updateCpuPctUi(readCpuPct());
      refreshRangesLog();
    } catch (_) {
      updateCpuPctUi(50);
    }
  }

  $("rangeStartEdit")?.addEventListener("input", () => {
    if ($("rangeStartEdit")) $("rangeStartEdit").dataset.dirty = "1";
  });
  $("rangeStep")?.addEventListener("input", () => {
    updateRangeStepHint($("rangeStep").value);
  });

  async function applyRangeStart(restart) {
    const start = $("rangeStartEdit")?.value?.trim();
    if (!start || start.length < 1) {
      toast("Hex de départ requis", "error");
      return;
    }
    try {
      const r = await api("/api/scan/ranges", {
        method: "POST",
        body: JSON.stringify({
          action: "set_start",
          start,
          restart: !!restart,
        }),
      });
      if ($("rangeStartEdit")) delete $("rangeStartEdit").dataset.dirty;
      if ($("startKey")) $("startKey").value = r.manual_start || start;
      toast(
        restart
          ? "Départ appliqué — scan relancé"
          : "Départ enregistré (prochain démarrage)",
        "success"
      );
      refreshRangesLog();
      persistScanConfig();
    } catch (e) {
      toast(e.message || String(e), "error");
    }
  }
  $("btnApplyStart")?.addEventListener("click", () => applyRangeStart(false));
  $("btnApplyStartRestart")?.addEventListener("click", () => applyRangeStart(true));
  $("btnApplyStep")?.addEventListener("click", async () => {
    const range_step =
      parseInt($("rangeStep")?.value || "1073741824", 10) || 1073741824;
    try {
      await api("/api/scan/ranges", {
        method: "POST",
        body: JSON.stringify({ action: "set_step", range_step }),
      });
      updateRangeStepHint(range_step);
      toast(`Pas enregistré : ${formatNumber(range_step)} clés`, "success");
      persistScanConfig();
    } catch (e) {
      toast(e.message || String(e), "error");
    }
  });
  $("scanCpuPct")?.addEventListener("input", () => {
    updateCpuPctUi($("scanCpuPct").value);
    persistScanConfig();
  });
  $("scanCpuPctNum")?.addEventListener("change", () => {
    updateCpuPctUi($("scanCpuPctNum").value);
    persistScanConfig();
  });
  $("threads")?.addEventListener("change", () => {
    updateCpuPctUi(readCpuPct());
    persistScanConfig();
  });

  $("scanForm")?.addEventListener("submit", async (e) => {
    e.preventDefault();
    const transforms = selectedTransforms();
    renderActiveTransforms(transforms);
    const body = buildScanConfigBody();
    try {
      const r = await api("/api/scan/start", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const cores = logicalCores();
      const w =
        body.threads > 0
          ? body.threads
          : body.cpu_pct === 0
            ? 0
            : Math.max(1, Math.floor((cores * body.cpu_pct) / 100));
      setMsg(
        "scanMsg",
        `PID ${r.pid} · CPU ${w} thr (${body.cpu_pct}%) · ${transforms.join(", ")}`,
        "success"
      );
      toast(`Scan démarré · ${w} workers CPU`, "success");
    } catch (err) {
      setMsg("scanMsg", err.message, "error");
      toast(err.message, "error");
    }
  });
  $("btnStop")?.addEventListener("click", async () => {
    await api("/api/scan/stop", { method: "POST", body: "{}" });
    setMsg("scanMsg", "Arrêt demandé");
  });
  loadScanConfig();

  // Live transform preview when editing checkboxes (scan stopped)
  document.querySelectorAll("#transformChecks input").forEach((el) => {
    el.addEventListener("change", () => {
      if (!$("btnStart")?.disabled) renderActiveTransforms(selectedTransforms());
    });
  });

  // WS
  let wsRetry = 0;
  function connectWs() {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(`${proto}//${location.host}/ws`);
    ws.onopen = () => {
      wsRetry = 0;
      if ($("wsStatus")) $("wsStatus").className = "ws-status connected";
      setText("wsLabel", "Live");
    };
    ws.onclose = () => {
      if ($("wsStatus")) $("wsStatus").className = "ws-status error";
      setText("wsLabel", "…");
      setTimeout(connectWs, Math.min(1000 * Math.pow(1.5, wsRetry++), 15000));
    };
    ws.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data);
        if (msg.type === "tick") {
          if (msg.scan) updateScan(msg.scan);
          if (msg.dict) updateDict(msg.dict);
          if (msg.bitcoind) updateBtc(msg.bitcoind);
        }
      } catch (_) {}
    };
  }
  connectWs();

  // boot
  setHuntMethods(true);
  setDictMethods(true);
  renderActiveTransforms(selectedTransforms());
  loadCorpora();
  renderFound();
  refreshBtc();
  refreshSnap();
  refreshHealth();
  refreshScan();
  setInterval(refreshHealth, 12000);
  setInterval(refreshScan, 3000);
  refreshRangesLog();
  setInterval(refreshRangesLog, 15000);
  // Scan listes: 2 mises à jour / seconde
  setInterval(async () => {
    try {
      updateDict(await api("/api/dict/status"));
    } catch (_) {}
  }, 500);
})();
