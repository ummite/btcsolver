/**
 * BTC Solver — UI focused on "finding keys with balance"
 */
(() => {
  "use strict";

  const $ = (id) => document.getElementById(id);
  const FOUND_KEY = "btcsolver_found_keys_v1";
  /** Explorateur public — address BTC uniquement, JAMAIS de clé privée dans l’URL */
  const BLOCKCHAIN_EXPLORER_ADDR =
    "https://www.blockchain.com/explorer/addresses/btc/";
  let lastHits = [];

  // ── Alert System ────────────────────────────────────────────────────────
  const ALERT_STORAGE = "btcsolver_alerts_v1";
  let alertConfig = loadAlertConfig();
  let lastAlertState = {};

  function loadAlertConfig() {
    try {
      const saved = localStorage.getItem(ALERT_STORAGE);
      if (saved) return JSON.parse(saved);
    } catch (_) {}
    return {
      on_match: true,
      on_crash: true,
      on_gpu_drop: true,
      gpu_drop_threshold: 30,
      on_utxo_stale: true,
      utxo_stale_hours: 18,
      sound_enabled: true,
      browser_notification: false,
    };
  }

  function saveAlertConfig() {
    try { localStorage.setItem(ALERT_STORAGE, JSON.stringify(alertConfig)); } catch (_) {}
  }

  function checkAlerts(scanStats, health) {
    if (!scanStats) return;
    if (alertConfig.on_match) {
      const hits = scanStats.matches_found || 0;
      const prevHits = lastAlertState.hits || 0;
      if (hits > prevHits && hits > 0) {
        triggerAlert("⚡ MATCH FOUND", `Key with balance detected! Total: ${hits}`, "success");
        lastAlertState.hits = hits;
      }
    }
    if (alertConfig.on_crash) {
      const wasRunning = lastAlertState.running;
      const isRunning = scanStats.running;
      if (wasRunning && !isRunning) {
        triggerAlert("🛑 SCAN STOPPED", "Brute-force scan has stopped unexpectedly", "error");
      }
      lastAlertState.running = isRunning;
    }
    if (alertConfig.on_gpu_drop && scanStats.running) {
      const gpuUtil = scanStats.gpu_util || 0;
      if (gpuUtil > 0 && gpuUtil < alertConfig.gpu_drop_threshold) {
        const key = `gpu_drop_${Math.floor(Date.now() / 60000)}`;
        if (lastAlertState[key]) return;
        lastAlertState[key] = true;
        triggerAlert("⚠️ GPU LOW", `GPU utilization dropped to ${gpuUtil}%`, "warning");
      }
    }
    if (alertConfig.on_utxo_stale && health) {
      const utxoAge = health.utxo_lag_hours || 0;
      if (utxoAge > alertConfig.utxo_stale_hours) {
        const key = `utxo_stale_${Math.floor(Date.now() / 300000)}`;
        if (lastAlertState[key]) return;
        lastAlertState[key] = true;
        triggerAlert("⏰ UTXO STALE", `UTXO index is ${utxoAge.toFixed(1)}h behind tip`, "warning");
      }
    }
  }

  function triggerAlert(title, message, type) {
    if (alertConfig.browser_notification && Notification.permission === "granted") {
      try { new Notification(title, { body: message }); } catch (_) {}
    }
    if (alertConfig.sound_enabled && type === "success") {
      try {
        const ac = new (window.AudioContext || window.webkitAudioContext)();
        const osc = ac.createOscillator();
        const gain = ac.createGain();
        osc.connect(gain); gain.connect(ac.destination);
        osc.frequency.value = 880; gain.gain.value = 0.3;
        osc.start();
        gain.gain.exponentialRampToValueAtTime(0.001, ac.currentTime + 0.5);
        osc.stop(ac.currentTime + 0.5);
      } catch (_) {}
    }
    toast(`${title}: ${message}`, type);
  }

  /** Block lag → human-readable time estimate (10 min/block average) */
  function formatLagTime(blocks) {
    if (!blocks || blocks <= 0) return "";
    const hours = (blocks * 10) / 60;
    if (hours >= 24) return ` (~${formatNumber(hours)}h / ~${(hours / 24).toFixed(1)}d)`;
    return ` (~${hours.toFixed(1)}h)`;
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
   * Exact integer (UTXO / Core block heights) — never K/M shortcut.
   * en-US locale for readability: 935000 → "935,000"
   */
  function formatHeight(n) {
    if (n == null || n === "" || Number.isNaN(Number(n))) return "—";
    return Math.round(Number(n)).toLocaleString("en-US");
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

  /** Copy button to the right of a value (address / hex). */
  function copyBtn(text, label = "Copy", kind = "text") {
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
        e.preventFromfault();
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
            ? "Public key copied"
            : kind === "priv"
              ? "Private key copied"
              : kind === "addr"
                ? "Address copied"
                : "Copied";
        try {
          await copyTextToClipboard(t);
          toast(okMsg, "success");
        } catch (_) {
          toast("Cannot copy", "error");
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
    // Affichage court pour les longs hex / addresss — copie = valeur complète
    const display =
      full === ""
        ? "—"
        : opts.short !== false && full.length > 28
          ? shortDisplay(full, opts.head || 12, opts.tail || 10)
          : full;
    const btnLabel =
      kind === "pub"
        ? "Copy pub"
        : kind === "priv"
          ? "Copy priv"
          : kind === "addr"
            ? "Copy addr"
            : "Copy";
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
      btn = `<button type="button" class="btn btn-ghost btn-sm btn-copy-inline" data-copy="" data-copy-kind="pub"${privAttr} title="Fromrive then copy compressed pub">Copy pub</button>`;
    }
    // Lien explorateur uniquement pour les addresss publiques (jamais priv / pub hex)
    const explorer =
      kind === "addr" && isBtcPublicAddress(full)
        ? explorerLinkHtml(full, "Check balance")
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

  /** Vraie address BTC publique (legacy / P2SH / bech32) — pas de priv / WIF / hex. */
  function isBtcPublicAddress(a) {
    const s = String(a || "").trim();
    if (!s || s === "—") return false;
    // Refuse explicitement tout ce qui ressemble à une clé privée
    if (/^[0-9a-fA-F]{64}$/.test(s)) return false;
    if (/^[5KL][1-9A-HJ-NP-Za-km-z]{50,51}$/.test(s)) return false; // WIF
    return /^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$|^bc1[a-z0-9]{25,90}$/i.test(s);
  }

  /**
   * URL blockchain.com pour une public address uniquement.
   * Ne jamais y coller priv / WIF / seed.
   */
  function explorerAddrUrl(addr) {
    if (!isBtcPublicAddress(addr)) return "";
    return BLOCKCHAIN_EXPLORER_ADDR + encodeURIComponent(String(addr).trim());
  }

  /** Lien « vérifier solde » — target=_blank, rel=noopener (public address seule). */
  function explorerLinkHtml(addr, label = "Check balance") {
    const url = explorerAddrUrl(addr);
    if (!url) return "";
    return `<a class="btn btn-explorer btn-sm" href="${esc(url)}" target="_blank" rel="noopener noreferrer" title="Opens blockchain.com with PUBLIC address only — no private key is sent">${esc(label)} ↗</a>`;
  }

  /**
   * Adresse publique « format court » pour explorers :
   * priorité à l’address du hit, sino la plus courte parmi legacy / segwit / wrapped / taproot.
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
    // Si le hit est une vraie address BTC, on le garde (c’est celle du solde trouvé)
    if (hit && isBtcPublicAddress(hit)) {
      return hit;
    }
    // Sino la plus courte (format « court ») parmi les addresss valides
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
   * Vault bar: PRIV (red) + short public ADDR (green) + compressed PUB (light green).
   */
  function vaultKeyActions(priv, addr, pub) {
    const p = priv || "";
    const a = addr || "";
    const u = ensureCompressedPub(pub) || pub || "";
    const explor = explorerLinkHtml(a, "Check balance on-chain");
    return `<div class="vault-key-actions">
      <button type="button" class="btn btn-copy-priv-lg" data-copy="${esc(p)}" data-copy-kind="priv" ${
        p ? "" : "disabled"
      } title="Copy private key (hex 64) — stay local, never paste into a web browser">⬛ Copy PRIV</button>
      <button type="button" class="btn btn-copy-addr-lg" data-copy="${esc(a)}" data-copy-kind="addr" ${
        a ? "" : "disabled"
      } title="Copy public address (short format for explorers)">🟩 Copy ADDR</button>
      <button type="button" class="btn btn-copy-pub-lg" data-copy="${esc(u)}" data-copy-kind="pub" data-priv="${esc(
      p
    )}" ${p || u ? "" : "disabled"} title="Short compressed public key (02/03…, 33 bytes)">Copy PUB hex</button>
      ${explor || `<span class="hint" style="align-self:center">no public address to explore</span>`}
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

  /**
   * Update the scan toggle button (#pillScan) with state, text, and tooltip.
   * state: "on" | "off" | "error"
   * errorReason: optional string shown in the tile error element
   */
  function setScanPill(state, text, title, errorReason) {
    const el = $("pillScan");
    if (!el) return;
    el.textContent = text;
    el.className = "scan-toggle-btn is-" + state;
    el.title = title != null && title !== "" ? String(title) : String(text || "");
    const errEl = $("infoScanError");
    if (errEl) {
      errEl.textContent = errorReason || "";
    }
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
        throw new Error(`Invalid JSON (${path}): ${text.slice(0, 120)}`);
      }
      // Only treat TOP-LEVEL API failures as errors (not nested bitcoind.message)
      if (!res.ok) {
        throw new Error(data.error || data.message || `HTTP ${res.status} ${path}`);
      }
      if (data.success === false) {
        throw new Error(data.error || data.message || "API failure");
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
      if (btn.dataset.tab === "strategies") {
        renderStrategies();
        renderPatternAnalysis();
        renderWatchlist();
        renderBrainwalletPatterns();
        renderQuickActions();
      }
    });
  });

  // ── Clock ───────────────────────────────────────────────────────────────
  function tickClock() {
    const now = new Date();
    if ($("clock"))
      $("clock").textContent = now.toLocaleTimeString("en-US", { hour12: false });
    if ($("clockUtc"))
      $("clockUtc").textContent = now.toLocaleTimeString("en-US", { hour12: false, timeZone: "UTC" }) + " (UTC)";
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
      el.innerHTML = "block — · --:--:--";
      return;
    }
    const height = btc.blocks != null ? formatHeight(btc.blocks) : "—";
    const blockDt = new Date(Number(btc.block_time) * 1000);
    const blockStr = blockDt.toLocaleTimeString("en-US", { hour12: false });
    const now = Date.now() / 1000;
    const lagSec = now - Number(btc.block_time);
    let lagStr = "";
    if (lagSec < 60) lagStr = `${Math.round(lagSec)}s ago`;
    else if (lagSec < 3600) lagStr = `${Math.round(lagSec / 60)}min ago`;
    else lagStr = `${Math.round(lagSec / 3600)}h${Math.round((lagSec % 3600) / 60)} ago`;
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
    box.innerHTML = `<p class="hint">Preparing public keys…</p>`;
    await enrichFoundPubkeys();
    const list = loadFound().map(normalizeVaultEntry);
    if (!list.length) {
      box.innerHTML =
        "No keys saved. Run a test that finds a balance, then « Save hits ». Pubkeys are auto-derived to paste into an explorer.";
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
          <div class="result-row"><span class="label">input</span><span class="value">${esc(m.input || m.phrase || "")}</span></div>
          ${rowCopy("public address", addrShort, "addr", { force: true, short: false })}
          ${rowCopy("priv hex", priv, "priv", { force: true, head: 10, tail: 8 })}
          ${rowCopy("pub hex (short comp)", pub, "pub", { force: true, priv, head: 10, tail: 8 })}
          ${m.addresses?.legacy && m.addresses.legacy !== addrShort ? rowCopy("legacy", m.addresses.legacy, "addr", { short: false }) : ""}
          ${m.addresses?.segwit && m.addresses.segwit !== addrShort ? rowCopy("segwit", m.addresses.segwit, "addr", { short: false }) : ""}
          ${m.addresses?.wrapped && m.addresses.wrapped !== addrShort ? rowCopy("wrapped", m.addresses.wrapped, "addr", { short: false }) : ""}
          ${m.addresses?.taproot && m.addresses.taproot !== addrShort ? rowCopy("taproot", m.addresses.taproot, "addr", { short: false }) : ""}
          <div class="result-row"><span class="label">saved</span><span class="value">${esc(m.saved_at || "")}</span></div>
          <button type="button" class="btn btn-ghost btn-sm" data-rm="${i}">Remove</button>
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
    toast("MAX mode activated", "success");
  });
  $("btnModeFast")?.addEventListener("click", () => {
    setHuntMethods(false);
    toast("Fast mode", "");
  });
  $("btnDictMax")?.addEventListener("click", () => setDictMethods(true));
  $("btnDictFast")?.addEventListener("click", () => setDictMethods(false));
  $("btnClearKey")?.addEventListener("click", () => {
    if ($("keyInput")) $("keyInput").value = "";
    if ($("keyResult")) $("keyResult").innerHTML = "No test yet.";
  });

  // ── Key check (single or multi-line batch) ──────────────────────────────
  function renderKeyResults(data) {
    const box = $("keyResult");
    if (!box) return;
    if (data.error) {
      box.innerHTML = `<div class="result-row"><span class="label">Error</span><span class="value">${esc(data.error)}</span></div>`;
      return;
    }
    const results = data.results || [];
    lastHits = results.filter((r) => (r.total_balance_sats || 0) > 0);
    if (!results.length) {
      box.innerHTML = `<p class="hint">0 hit shown (candidates: ${data.candidates || 0}). Uncheck « Only balances > 0 » to see zero-balance addresses.</p>`;
      return;
    }
    let html = `<p class="hint">${data.candidates || results.length} candidates · total ${data.total_balance_btc || 0} BTC · hits ${lastHits.length}</p>`;
    for (const r of results.slice(0, 60)) {
      const bal = r.total_balance_sats > 0;
      const priv = r.privkey_hex || "";
      const pub = r.pubkey_hex || "";
      html += `<div class="match-item" style="${bal ? "" : "background:var(--bg-input);border-color:var(--border)"}">`;
      html += `<div class="method-tag">${esc(r.method || r.input_format)}</div>`;
      if (bal)
        html += `<div class="result-balance">${r.total_balance_btc} BTC (${formatNumber(r.total_balance_sats)} sats)</div>`;
      else html += `<div class="type">0 BTC</div>`;
      html += `<div class="result-row"><span class="label">input</span><span class="value hex-full">${esc(r.input)}</span></div>`;
      html += rowCopy("priv hex (64)", priv, "priv");
      // Clé publique compressée (hex) — bouton dédié « Copier pub » (pas la privée)
      html += rowCopy("public key (pub hex)", pub || "", "pub");
      if (r.addresses) {
        html += rowCopy("legacy", r.addresses.legacy, "addr");
        html += rowCopy("segwit", r.addresses.segwit, "addr");
        if (r.addresses.wrapped) html += rowCopy("wrapped", r.addresses.wrapped, "addr");
        html += rowCopy("taproot", r.addresses.taproot, "addr");
      }
      if (r.matches?.length) {
        for (const m of r.matches) {
          html += `<div class="type">${esc(m.address_type)} · ${m.value_btc} BTC</div>`;
          html += rowCopy("address", m.address, "addr");
        }
      }
      html += `</div>`;
    }
    box.innerHTML = html;
    box.className = "key-result" + (data.total_balance_sats > 0 ? " has-balance" : "");
    wireCopyButtons(box);
  }

  $("keyForm")?.addEventListener("submit", async (e) => {
    e.preventFromfault();
    const raw = ($("keyInput")?.value || "").trim();
    if (!raw) return setMsg("keyMsg", "Empty", "error");
    const lines = raw
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter((l) => l && !l.startsWith("#"));
    setMsg("keyMsg", lines.length > 1 ? `Batch of ${lines.length}…` : "Testing…");
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
        `${data.candidates || data.count || 0} candidates · ${hits.length} hit(s)`,
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
    toast(n ? `${n} key(s) added to vault` : "Nothing new to save", n ? "success" : "");
    renderFound();
  });
  $("btnExportFound")?.addEventListener("click", () => {
    const blob = new Blob([JSON.stringify(loadFound(), null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `btcsolver-found-${Date.now()}.json`;
    a.click();
  });

  // ── Export scan stats (JSON) ────────────────────────────────────────────
  $("btnExportScanStats")?.addEventListener("click", async () => {
    try {
      toast("Downloading scan export…", "");
      const res = await fetch("/api/scan/export");
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const blob = await res.blob();
      const a = document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = `btcsolver-scan-export-${Date.now()}.json`;
      a.click();
      toast("Scan stats exported", "success");
    } catch (e) {
      toast("Export failed: " + e.message, "error");
    }
  });

  // ── Export archive (CSV) ───────────────────────────────────────────────
  const exportArchiveFn = async () => {
    try {
      toast("Downloading archive CSV…", "");
      const res = await fetch("/api/keys/archive/export");
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const blob = await res.blob();
      const a = document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = `btcsolver-keys-archive-${Date.now()}.csv`;
      a.click();
      toast("Archive exported", "success");
    } catch (e) {
      toast("Export failed: " + e.message, "error");
    }
  };
  $("btnExportArchive")?.addEventListener("click", exportArchiveFn);
  $("btnExportArchiveVault")?.addEventListener("click", exportArchiveFn);

  // ── Pause / Resume scan ────────────────────────────────────────────────
  $("btnScanPause")?.addEventListener("click", async () => {
    try {
      const r = await api("/api/scan/pause", { method: "POST", body: "{}" });
      if (r.success) {
        window.__scanPaused = true;
        toast(`Scan paused${r.position_saved ? " (position saved)" : ""}`, "");
        if ($("btnScanPause")) $("btnScanPause").disabled = true;
        if ($("btnScanResume")) $("btnScanResume").disabled = false;
        setScanPill("off", "PAUSED", "Scan paused — click Resume to continue", "");
        // Also update the tile pause panel
        if (window.__updatePausePanel) window.__updatePausePanel(false);
      } else {
        toast(r.error || "Pause failed", "error");
      }
    } catch (e) {
      toast("Pause error: " + e.message, "error");
    }
  });

  $("btnScanResume")?.addEventListener("click", async () => {
    try {
      const r = await api("/api/scan/resume", { method: "POST", body: "{}" });
      if (r.success) {
        window.__scanPaused = false;
        const from = r.resumed_from ? ` from ${shortenHex(r.resumed_from, 8, 6)}` : "";
        toast(`Scan resumed${from}`, "success");
        if ($("btnScanPause")) $("btnScanPause").disabled = false;
        if ($("btnScanResume")) $("btnScanResume").disabled = true;
        // Also update the tile pause panel
        if (window.__updatePausePanel) window.__updatePausePanel(true);
      } else {
        toast(r.error || "Resume failed", "error");
      }
    } catch (e) {
      toast("Resume error: " + e.message, "error");
    }
  });

  // ── Pause panel — duration selector + countdown + auto-resume ──────────
  (function initPausePanel() {
    const btnPause      = $("btnScanPauseTile");
    const btnResume     = $("btnScanResumeTile");
    const durSelector   = $("scanPauseDurations");
    const pauseSelector = $("scanPauseSelector");
    const pauseActive   = $("scanPauseActive");
    const panel         = $("scanPausePanel");
    const countdownEl   = $("scanPauseCountdown");

    let pauseTimerId    = null;   // setInterval for countdown
    let autoResumeId    = null;   // setTimeout for auto-resume
    let pauseDeadline   = null;   // Date when auto-resume should fire

    // Format milliseconds remaining into a readable string
    function fmtRemaining(ms) {
      if (!ms || ms < 0) return "∞";
      const totalSec = Math.floor(ms / 1000);
      const h  = Math.floor(totalSec / 3600);
      const m  = Math.floor((totalSec % 3600) / 60);
      const s  = totalSec % 60;
      if (h > 0) return `${h}h ${String(m).padStart(2,'0')}:${String(s).padStart(2,'0')}`;
      return `${String(m).padStart(2,'0')}:${String(s).padStart(2,'0')}`;
    }

    // Show/hide the panel based on scan state
    function updatePausePanelVisibility(scanRunning) {
      if (!panel) return;
      if (scanRunning && !window.__scanPaused) {
        panel.style.display = "flex";
        pauseSelector.style.display = "flex";
        pauseActive.style.display = "none";
        durSelector.style.display = "none";
      } else if (window.__scanPaused) {
        panel.style.display = "flex";
        pauseSelector.style.display = "none";
        pauseActive.style.display = "flex";
      } else {
        panel.style.display = "none";
      }
    }

    // Toggle duration selector visibility
    btnPause?.addEventListener("click", () => {
      if (durSelector.style.display === "flex") {
        durSelector.style.display = "none";
      } else {
        durSelector.style.display = "flex";
      }
    });

    // Duration button clicks
    durSelector?.addEventListener("click", async (e) => {
      const btn = e.target.closest(".btn-pause-dur");
      if (!btn) return;
      const hours = parseInt(btn.dataset.hours) || 0;

      // Hide selector
      durSelector.style.display = "none";

      // Call pause API
      try {
        const r = await api("/api/scan/pause", { method: "POST", body: "{}" });
        if (!r.success) {
          toast(r.error || "Pause failed", "error");
          return;
        }

        window.__scanPaused = true;
        setScanPill("off", "PAUSED", "Scan paused — click Resume to continue", "");

        // Set auto-resume timer
        if (hours > 0) {
          pauseDeadline = new Date(Date.now() + hours * 3600 * 1000);
          // Auto-resume after duration
          autoResumeId = setTimeout(async () => {
            await doResume();
          }, hours * 3600 * 1000);
          // Start countdown
          pauseTimerId = setInterval(() => {
            const remaining = pauseDeadline ? pauseDeadline.getTime() - Date.now() : 0;
            countdownEl.textContent = fmtRemaining(remaining);
            if (remaining <= 0) {
              clearInterval(pauseTimerId);
              pauseTimerId = null;
            }
          }, 1000);
        } else {
          pauseDeadline = null;
          countdownEl.textContent = "∞";
        }

        // Show active state
        updatePausePanelVisibility(false);
        toast(`Scan paused${hours > 0 ? ` — auto-resume in ${hours}h` : " (indéfini)"}`, "");
      } catch (err) {
        toast("Pause error: " + err.message, "error");
      }
    });

    // Resume button handler
    async function doResume() {
      try {
        const r = await api("/api/scan/resume", { method: "POST", body: "{}" });
        if (r.success) {
          window.__scanPaused = false;
          const from = r.resumed_from ? ` from ${shortenHex(r.resumed_from, 8, 6)}` : "";
          toast(`Scan resumed${from}`, "success");
          // Clear timers
          if (pauseTimerId) { clearInterval(pauseTimerId); pauseTimerId = null; }
          if (autoResumeId) { clearTimeout(autoResumeId); autoResumeId = null; }
          pauseDeadline = null;
          updatePausePanelVisibility(true);
        } else {
          toast(r.error || "Resume failed", "error");
        }
      } catch (err) {
        toast("Resume error: " + err.message, "error");
      }
    }

    btnResume?.addEventListener("click", doResume);

    // Expose globally so other code can trigger panel updates
    window.__updatePausePanel = updatePausePanelVisibility;
  })();

  // ── Scan Easy Keys (Background Corpus) ─────────────────────────────────
  let corpusPollInterval = null;

  $("btnScanEasyKeys")?.addEventListener("click", async () => {
    try {
      toast("Starting corpus scan in background…", "");
      const r = await api("/api/scan/easy-keys", {
        method: "POST",
        body: JSON.stringify({ use_gpu: false }),
        timeoutMs: 60000,
      });
      if (r.success) {
        window.__corpusRunning = true;
        toast(`Corpus scan started: ${r.total_keys?.toLocaleString()} keys in ${r.files} files`, "success");
        // Start polling progress
        if (corpusPollInterval) clearInterval(corpusPollInterval);
        updateCorpusProgress();
        corpusPollInterval = setInterval(updateCorpusProgress, 3000);
      } else {
        toast(r.error || "Corpus scan failed", "error");
      }
    } catch (e) {
      toast("Corpus scan error: " + e.message, "error");
    }
  });

  async function updateCorpusProgress() {
    try {
      const r = await api("/api/scan/corpus/progress");
      const el = $("corpusProgress");
      if (el) {
        el.style.display = 'block';
        const pct = (r.progress_pct || 0).toFixed(1);
        const tested = (r.keys_tested || 0).toLocaleString();
        const total = (r.keys_total || 0).toLocaleString();
        const matches = r.matches_found || 0;
        const statusText = $("corpusStatusText");
        if (statusText) {
          statusText.innerHTML = `
            <strong>Corpus Scan</strong> ${r.running ? '⏳ Running' : '✅ Done'}
            <div class="progress-bar"><div class="progress-fill" style="width:${pct}%"></div></div>
            <span>${tested} / ${total} keys (${pct}%) — ${matches} match(es)</span>
            ${r.status ? `<br><small>${r.status}</small>` : ''}`;
        }
      }
      if (!r.running && corpusPollInterval) {
        clearInterval(corpusPollInterval);
        corpusPollInterval = null;
        window.__corpusRunning = false;
        toast(`Corpus scan complete: ${r.keys_tested?.toLocaleString()} keys, ${matches} match(es)`, matches > 0 ? "success" : "");
      }
    } catch (_) { /* ignore */ }
  }

  $("btnStopCorpus")?.addEventListener("click", async () => {
    try {
      const r = await api("/api/scan/corpus/stop", { method: "POST" });
      if (r.success) {
        window.__corpusRunning = false;
        toast("Corpus scan stopping…", "");
      }
    } catch (e) { toast("Stop error: " + e.message, "error"); }
  });
  $("btnClearFound")?.addEventListener("click", () => {
    if (confirm("Empty the local vault?")) {
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
        o.textContent = `${c.name} (${c.size_mb} MB)`;
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
          ${rowCopy("address", m.address, "addr")}
          ${m.pubkey_hex ? rowCopy("public key", m.pubkey_hex, "pub") : ""}
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
    const warn = count > 100000 ? " ⚠ large" : count > 10000 ? " · medium" : " · light";
    el.textContent = `~${count.toLocaleString("en-US")} base phrases (n=${n} words)${warn} + hashes`;
  }

  /** Estimation affixes tous-caractères (94 ASCII) par phrase de base. */
  function estimateAffix() {
    const el = $("dAffixEst");
    if (!el) return;
    const pre = Math.min(2, Math.max(0, parseInt($("dCharPrefixLen")?.value || "0", 10) || 0));
    const suf = Math.min(2, Math.max(0, parseInt($("dCharSuffixLen")?.value || "0", 10) || 0));
    if (pre === 0 && suf === 0) {
      el.textContent = "affixes: off";
      return;
    }
    const C = 94;
    const nPre = pre === 0 ? 1 : pre === 1 ? C : C * C;
    const nSuf = suf === 0 ? 1 : suf === 1 ? C : C * C;
    const perBase = nPre * nSuf;
    const note =
      perBase > 5e6
        ? " · very long (streaming, no ceiling)"
        : perBase > 50000
          ? " · long (streaming loops)"
          : " · ok";
    el.textContent = `affixes: ~${perBase.toLocaleString("en-US")} × bases × hashes / phrase (pref ${pre} × suf ${suf})${note}`;
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
    e.preventFromfault();
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
    setMsg("dictMsg", "Starting…");
    try {
      await api("/api/dict/start", { method: "POST", body: JSON.stringify(body) });
      setMsg("dictMsg", "Scan started", "success");
      toast("Dictionary scan started", "success");
    } catch (err) {
      setMsg("dictMsg", err.message, "error");
      toast(err.message, "error");
    }
  });
  $("btnDictStop")?.addEventListener("click", async () => {
    await api("/api/dict/stop", { method: "POST", body: "{}" });
    setMsg("dictMsg", "Stop requested");
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

    // Scan toggle button: ON/OFF/PAUSED
    if (window.__scanPaused) {
      setScanPill("off", "PAUSED", "Scan paused — click to resume or use Resume button", "");
    } else if (anyScanOn || totalRate > 0) {
      setScanPill("on", "SCAN ON", `${formatCompact(totalRate)}/s · ${formatNumber(totalTested)} tested`);
    } else {
      setScanPill("off", "SCAN OFF", "Background scan stopped");
    }

    // Update pause panel visibility
    if (window.__updatePausePanel) {
      window.__updatePausePanel(anyScanOn || totalRate > 0);
    }

    // Update "Scan en cours" tile with GPU/CPU breakdown
    if ($("infoScanRate")) {
      $("infoScanRate").textContent = totalRate > 0 ? `${formatCompact(totalRate)} /s` : "— /s";
    }
    if ($("infoScanTested")) {
      $("infoScanTested").textContent = totalTested > 0 ? `tested: ${formatNumber(totalTested)}` : "tested: —";
    }
    if ($("infoScanMode")) {
      const modeLabel = [];
      const scanMode = scanData.mode || "sequential";
      const gpuCount = (scanData.gpu_rates || []).filter(g => (g.keys_per_sec || g.keys_per_sec_avg || 0) > 0).length;
      const cpuThreads = scanData.cpu_threads || 0;

      // Brute-force scan type
      if (bruteOn) {
        let label = "Auto Scan";
        if (scanMode === "random") label = "Random Scan";
        const hw = [];
        if (gpuCount > 0) hw.push(`${gpuCount} GPU`);
        if (cpuThreads > 0) hw.push(`${cpuThreads} CPU`);
        if (hw.length) label += ` (${hw.join(" + ")})`;
        modeLabel.push(label);
      }

      // Dict scan
      if (dictOn) {
        let label = "Dict Scan";
        const phrases = dictData.phrases_total || 0;
        const progress = dictData.progress_pct || 0;
        if (phrases > 0) label += ` (${progress.toFixed(1)}%)`;
        modeLabel.push(label);
      }

      // Corpus scan (check via global state)
      if (window.__corpusRunning) {
        modeLabel.push("Corpus Scan");
      }

      // Benchmark
      if (window.__benchRunning) {
        modeLabel.push("Benchmark");
      }

      $("infoScanMode").textContent = modeLabel.length ? `▶ ${modeLabel.join(" + ")}` : "mode: —";
    }
    // GPU/CPU detail line in the scan section
    if ($("scanRateDetail")) {
      const detailParts = [...gpuParts];
      if (cpuTotal > 0) detailParts.push(`CPU: ${formatCompact(cpuTotal)}/s`);
      $("scanRateDetail").textContent = detailParts.length ? detailParts.join(" · ") : "—";
    }

    // Error display in scan tile
    if ($("infoScanError")) {
      const scanErr = scanData.error || scanData.message;
      const dictErr = dictData.error || dictData.message;
      const errMsg = scanErr || dictErr || "";
      $("infoScanError").textContent = errMsg;
    }

    // GPU/CPU vertical breakdown with thread config
    renderScanDeviceBreakdown(scanData, dictData, cpuTotal);

    if (synced) setPill("pillSync", "Chain OK", "ok", "Chain up to date");
    else if (running) setPill("pillSync", "Chain syncing", "warn", s?.simple_status || "Sync in progress");
    else setPill("pillSync", "Chain off", "warn", "Core stopped");

    if (indexOk) {
      const n = health.index_scripts || 0;
      setPill(
        "pillReady",
        `Idx ${formatCompact(n)}`,
        "ok",
        `Index loaded · ${formatNumber(n)} scripts`
      );
    } else {
      setPill("pillReady", "Idx …", "warn", "Index loading");
    }

    const msg = s?.simple_status || s?.message || "Test words = tab 1";
    setText("statusBarMsg", msg);
    if ($("statusBarMsg")) $("statusBarMsg").title = msg;

    updateTipWarning(window.__lastSnap, s, window.__lastHealth);
  }

  /** UTXO valid for tests if within this many hours of tip (user rule). */
  const UTXO_VALID_HOURS = 24;       // < 24h = vert (OK)
  const UTXO_WARNING_HOURS = 168;    // 24h–7j = jaune (acceptable), > 7j = rouge

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
    // Warning (yellow): 24h – 7 days behind tip (acceptable but not ideal)
    const warningState =
      lag.hours != null && lag.hours > UTXO_VALID_HOURS && lag.hours <= UTXO_WARNING_HOURS;
    // Red: > 7 days behind tip (stale)
    const staleState = !validForTests && !warningState;
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
          `UTXO index block ${formatHeight(lag.idxH)} / Core tip ${formatHeight(tip)}` +
            (lagB != null ? ` (lag ${formatHeight(lagB)} blocks${formatLagTime(lagB)})` : "")
        );
      } else if (lag.idxH != null) {
        parts.push(
          `UTXO index: block ${formatHeight(lag.idxH)}${
            snap?.block_time_utc ? " · " + snap.block_time_utc : ""
          }`
        );
      } else {
        parts.push("UTXO index: unknown height");
      }
      if (coreBlocks != null) {
        parts.push(
          `Core: ${formatHeight(coreBlocks)} blocks` +
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
        if (validForTests) {
          parts.push(`lag ≈ ${hLabel} (< ${UTXO_VALID_HOURS}h → tests OK)`);
        } else if (warningState) {
          parts.push(`lag ≈ ${hLabel} (${UTXO_VALID_HOURS}h–${UTXO_WARNING_HOURS}h → acceptable, candidates)`);
        } else {
          parts.push(`lag ≈ ${hLabel} (> ${UTXO_WARNING_HOURS}h → stale, unreliable)`);
        }
      }
      if (lag.blockLag != null && lag.blockLag > 0) {
        parts.push(`${formatHeight(lag.blockLag)} blocks behind tip${formatLagTime(lag.blockLag)}`);
      }
      meta.textContent = parts.join("  ·  ");
    }

    const body = $("tipWarnBody");
    const strong = warn.querySelector("strong");
    warn.classList.remove("hidden");

    if (validForTests || atExactTip) {
      // ===== GREEN: < 24h =====
      warn.classList.add("is-tip");
      warn.classList.remove("tip-stale", "tip-warning");
      if (strong) {
        strong.textContent = atExactTip
          ? "✓ UTXO at tip — balances reliable for tests"
          : `✓ UTXO valid for tests (< ${UTXO_VALID_HOURS} h of tip)`;
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
          `UTXO index at block <strong>${idxL}</strong> · tip Core <strong>${tipL}</strong>. ` +
          `The index is <strong>less than ${UTXO_VALID_HOURS} h</strong> of estimated tip ` +
          `(lag ≈ <strong>${hLabel}</strong>). ` +
          `You can use it to test keys / brainwallets. ` +
          `Verify on-chain before any spending.`;
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
          `Index block ${formatHeight(lag.idxH)} · Core ${formatHeight(tipN)} · valid for tests (<${UTXO_VALID_HOURS}h)`
        );
      }
    } else if (warningState) {
      // ===== YELLOW: 24h – 7 days =====
      warn.classList.add("tip-warning");
      warn.classList.remove("is-tip", "tip-stale");
      if (strong) {
        strong.textContent = `⚡ UTXO acceptable (${UTXO_VALID_HOURS}h–${UTXO_WARNING_HOURS}h lag) — hits = candidates`;
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
          `UTXO index at block <strong>${idxL}</strong> · tip Core <strong>${tipL}</strong>. ` +
          `Lag ≈ <strong>${hLabel}</strong> (between <strong>${UTXO_VALID_HOURS} h</strong> and <strong>${UTXO_WARNING_HOURS} h</strong>). ` +
          `Index is <strong>acceptable</strong> — matches are <em>candidates</em> (some balances may have been spent since index was built). ` +
          `Verify on-chain before any spending. ` +
          `<br>Auto-refresh at tip via <code>Keep-Core-And-Utxo.ps1</code>.`;
      }
      // UTXO pill: yellow when warning
      if ($("pillUtxo") && lag.idxH != null) {
        const tipN = coreBlocks != null ? coreBlocks : lag.tipH;
        setPill(
          "pillUtxo",
          tipN != null
            ? `UTXO ${formatHeight(lag.idxH)} / ${formatHeight(tipN)}`
            : `UTXO ${formatHeight(lag.idxH)}`,
          "warn",
          `Lag ${lag.hours?.toFixed(1) ?? "?"}h (acceptable ${UTXO_VALID_HOURS}–${UTXO_WARNING_HOURS}h) · candidates only`
        );
      }
    } else {
      // ===== RED: > 7 days =====
      warn.classList.remove("is-tip", "tip-warning");
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
          strong.textContent = "⚠ Bitcoin Core stopped — auto-restarting…";
        } else if (
          coreIbd ||
          (coreBlocksStale != null &&
            coreHeadersStale != null &&
            coreBlocksStale < coreHeadersStale - 3)
        ) {
          strong.textContent = "⏳ Core syncing chain — UTXO tip pending (normal)";
        } else {
          const idxS = lag.idxH != null ? formatHeight(lag.idxH) : "?";
          const tipS =
            coreBlocksStale != null
              ? formatHeight(coreBlocksStale)
              : lag.tipH != null
                ? formatHeight(lag.tipH)
                : "?";
          strong.textContent = `⚠ UTXO block ${idxS} / Core ${tipS} — stale (> ${UTXO_WARNING_HOURS}h), unreliable`;
        }
      }
      if (body) {
        const hLabel =
          lag.hours != null ? `${lag.hours.toFixed(1)} h` : "unknown";
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
            "The watchdog <code>Keep-Core-And-Utxo</code> will restart bitcoind shortly. " +
            "If it stays stopped: run <code>Install-AlwaysOn.bat</code> or <code>Launch-BitcoinCore.ps1</code>.";
        } else if (
          coreIbd ||
          (coreBlocksStale != null &&
            coreHeadersStale != null &&
            coreBlocksStale < coreHeadersStale - 3)
        ) {
          html =
            `<strong>Bitcoin Core is running</strong> and catching up to tip: ` +
            `<strong>${formatHeight(coreBlocksStale)}</strong> / <strong>${formatHeight(coreHeadersStale)}</strong> blocks` +
            (pct != null ? ` · ~${pct.toFixed(2)} %` : "") +
            `. ` +
            `Current UTXO index: block <strong>${idxFull}</strong>. ` +
            `The offline UTXO index can be regenerated at tip <em>only</em> when Core has finished (IBD = false). ` +
            `Auto-refresh: task <code>BTCSolver-Core-Utxo</code> (dumptxoutset at tip). ` +
            `<br>In the meantime: tests = <em>candidates only</em> (lag UTXO ≈ <strong>${hLabel}</strong>). ` +
            `<strong>Do not stop bitcoind.</strong>`;
        } else {
          html =
            `UTXO index at block <strong>${idxFull}</strong> · tip Bitcoin Core <strong>${tipFull}</strong>` +
            ` · lag <strong>${lagFull}</strong> blocks (≈ <strong>${hLabel}</strong>).<br>` +
            `Thresholds: <strong>&lt; ${UTXO_VALID_HOURS}h</strong> = valid · <strong>${UTXO_VALID_HOURS}–${UTXO_WARNING_HOURS}h</strong> = candidates · <strong>> ${UTXO_WARNING_HOURS}h</strong> = stale. ` +
            `Current lag ≈ <strong>${hLabel}</strong> — index is <strong>stale</strong>. ` +
            `Hits are <em>candidates only</em> (many balances likely spent since index was built). ` +
            `<br>Auto-refresh at tip via <code>Keep-Core-And-Utxo.ps1</code>.`;
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
        running && rpc ? "Online" : running ? "Process" : "Stopped"
      );
    }
    // Hauteurs Core en entier exact (pas de K)
    setText("btcBlocks", s.blocks != null ? formatHeight(s.blocks) : "—");
    setText("btcHeaders", s.headers != null ? formatHeight(s.headers) : "—");
    if (s.blocks != null) window.__lastBtc = s;
    setText("btcPeers", s.connections != null ? s.connections : "—");
    setText(
      "btcIbd",
      s.initialblockdownload == null ? "—" : s.initialblockdownload ? "yes" : "no"
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
        ? `HIT — ${count} key found!`
        : `NEW HIT — ${count} key(s) total`;
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
        ? `⚡ HIT ! ${count} key with activity/balance`
        : `⚡ New hit — total ${count}`,
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
    // Priorité : keys avec solde ; sino count archive (activité)
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
          ? `${withBal} balance · ${actOnly} activity without balance · archive data/keys-archive.json`
          : "no key with on-chain activity yet";
    }
    const tile = document.querySelector(".hit-tile");
    if (tile) tile.classList.toggle("has-hits", archCount > 0);
    setPill(
      "pillHits",
      `Hits ${formatCompact(archCount)}`,
      archCount > 0 ? "ok" : "",
      archCount > 0
        ? `${formatNumber(archCount)} active key(s) (balance or history) — data/keys-archive.json`
        : "No key with on-chain activity"
    );
  }

  /**
   * Badges Core / UTXO dans la tuile index :
   * - Core : vert si bitcoind roule + tip frais; jaune si > 1 h de lag; rouge si stopé
   * - UTXO : vert si lag < 24 h ; jaune si ≥ 24 h mais rebuild en cours ;
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
          coreTip != null ? `Core slow · ${formatHeight(coreTip)}` : "Core slow",
          `Core tip from ${lagH} h ago (last block > 1h)`
        );
      } else {
        setBadge(
          badgeCore,
          "is-green",
          coreTip != null ? `Core ON · ${formatHeight(coreTip)}` : "Core ON",
          coreTip != null
            ? `Bitcoin Core running · tip block ${coreTip}`
            : "Bitcoin Core running"
        );
      }
    } else if (coreRunning === false) {
      setBadge(
        badgeCore,
        "is-red",
        "Core OFF",
        "Bitcoin Core stopped — balances/tip not reliable"
      );
    } else {
      setBadge(badgeCore, "is-muted", "Core · ?", "Core status unknown");
    }

    // --- UTXO ---
    // Priorité : rebuild en cours = jaune (même si très en lag)
    let utxoCls = "is-muted";
    let utxoTxt = "UTXO · ?";
    let utxoTitle = "UTXO freshness unknown";
    if (rebuildInProgress) {
      utxoCls = "is-yellow";
      utxoTxt =
        idxH != null
          ? `UTXO rebuild… · ${formatHeight(idxH)}`
          : "UTXO rebuild…";
      utxoTitle =
        "Index rebuild in progress (dumptxoutset / dump_to_flat) — not yet red";
    } else if (lagHours != null && Number.isFinite(lagHours)) {
      if (lagHours < UTXO_VALID_HOURS) {
        // GREEN: < 24h
        utxoCls = "is-green";
        utxoTxt =
          idxH != null
            ? `UTXO OK · ${formatHeight(idxH)}`
            : "UTXO OK (< 24 h)";
        utxoTitle = `Index up to date (< ${UTXO_VALID_HOURS} h of tip) · lag ≈ ${lagHours.toFixed(1)} h`;
      } else if (lagHours <= UTXO_WARNING_HOURS) {
        // YELLOW: 24h – 7 days
        utxoCls = "is-yellow";
        utxoTxt =
          idxH != null
            ? `UTXO ⚡ · ${formatHeight(idxH)}`
            : `UTXO ⚡ (${lagHours.toFixed(0)}h)`;
        utxoTitle =
          `Lag ≈ ${lagHours.toFixed(1)} h (acceptable ${UTXO_VALID_HOURS}–${UTXO_WARNING_HOURS}h) · candidates only` +
          (blockLag != null ? ` · ${formatHeight(blockLag)} blocks${formatLagTime(blockLag)}` : "");
      } else {
        // RED: > 7 days
        utxoCls = "is-red";
        utxoTxt =
          idxH != null
            ? `UTXO stale · ${formatHeight(idxH)}`
            : `UTXO stale (${lagHours.toFixed(0)}h)`;
        utxoTitle =
          `Lag ≈ ${lagHours.toFixed(1)} h (> ${UTXO_WARNING_HOURS}h = 7 days) — stale` +
          (blockLag != null ? ` · ${formatHeight(blockLag)} blocks${formatLagTime(blockLag)}` : "");
      }
    } else if (blockLag != null) {
      // fallback sans heures : ~144 blocks ≈ 24h, ~2016 blocks ≈ 7 days
      if (blockLag < 144) {
        utxoCls = "is-green";
        utxoTxt =
          idxH != null
            ? `UTXO OK · ${formatHeight(idxH)}`
            : "UTXO OK";
        utxoTitle = `Lag ${formatHeight(blockLag)} blocks${formatLagTime(blockLag)} (< ~24 h)`;
      } else if (blockLag < 2016) {
        utxoCls = "is-yellow";
        utxoTxt =
          idxH != null
            ? `UTXO ⚡ · ${formatHeight(idxH)}`
            : "UTXO ⚡";
        utxoTitle = `Lag ${formatHeight(blockLag)} blocks${formatLagTime(blockLag)} (~24h–7d)`;
      } else {
        utxoCls = "is-red";
        utxoTxt =
          idxH != null
            ? `UTXO stale · ${formatHeight(idxH)}`
            : "UTXO stale";
        utxoTitle = `Lag ${formatHeight(blockLag)} blocks${formatLagTime(blockLag)} (> ~7 days), stale`;
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

    // Lag en heures (pipeline always-on ou estimation)
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
      (snap.age_hours != null ? `file ~${Number(snap.age_hours).toFixed(1)} h` : null);
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
          `UTXO index = block ${height} · Bitcoin Core tip = block ${coreTip}` +
          (blockLag != null ? ` · lag ${formatHeight(blockLag)} blocks${formatLagTime(blockLag)}` : "");
      }
      setText(
        "infoUtxoVsCore",
        `index ${formatHeight(height)} · Core tip ${formatHeight(coreTip)}` +
          (blockLag != null && blockLag > 0
            ? ` · lag ${formatHeight(blockLag)} blocks${formatLagTime(blockLag)}`
            : blockLag === 0
              ? " · at tip"
              : "")
      );
    } else if (height != null) {
      setText("infoUtxoBlock", `block ${formatHeight(height)}`);
      if ($("infoUtxoBlock")) {
        $("infoUtxoBlock").title = `UTXO index = block ${height} (tip Core unknown)`;
      }
      setText("infoUtxoVsCore", "Core tip: (unknown — Core offline ?)");
    } else {
      setText("infoUtxoBlock", "block —");
      setText("infoUtxoVsCore", "Core tip: —");
    }

    setText("infoUtxoDate", blockDate ? `block date: ${blockDate}` : "block date: (unavailable — Core not at tip / reindex)");
    setText("infoUtxoBuilt", built ? `index generated: ${formatUtcLabel(built)}` : "index generated: —");
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
          ? "0 (at tip)"
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
      height != null ? `index block ${height}` : null,
      coreTip != null ? `Core tip ${coreTip}` : null,
      blockLag != null ? `lag ${formatHeight(blockLag)} blocks${formatLagTime(blockLag)}` : null,
      snap.block_time_utc || null,
      ageH != null ? `file age ${Number(ageH).toFixed(1)} h` : null,
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
    if (!s) return;
    if (s.error) {
      // Only show error on the toggle if dict is NOT running
      const dictRunning2 = !!(health?.dict?.running);
      if (!dictRunning2) {
        setScanPill("error", "SCAN FAILED", "Error: " + s.error, s.error);
      }
      // Always show error in the scan tile regardless
      const errEl2 = $("infoScanError");
      if (errEl2) errEl2.textContent = s.error;
      return;
    }
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
    updateHitsDisplay(hits, arch);
    setText("gpuUtil", s.gpu_util != null ? Number(s.gpu_util).toFixed(0) + "%" : "—");
    setText("currentPosition", s.current_position || s.range_end || "—");

    // Update GPU temperature pills
    updateGpuTemps(s.gpu_rates || []);

    // Update pause/resume button states
    if ($("btnScanPause")) $("btnScanPause").disabled = !run;
    if ($("btnScanResume")) {
      const posFileExists = !!(window.__scanPaused || false);
      $("btnScanResume").disabled = run || !posFileExists;
    }

    // Hex COMPLET (pas de raccourci …) pour From / To / curseur
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
        `${formatNumber(s.ranges_done)} range(s) logged`
      );
    }
    if ($("btnCopyRangeStart")) {
      $("btnCopyRangeStart").style.display = rs ? "inline-block" : "noe";
      $("btnCopyRangeStart").onclick = () => {
        if (rs) copyTextToClipboard(rs).then(() => toast("From copied", "success"));
      };
    }
    if ($("btnCopyRangeEnd")) {
      $("btnCopyRangeEnd").style.display = re ? "inline-block" : "noe";
      $("btnCopyRangeEnd").onclick = () => {
        if (re) copyTextToClipboard(re).then(() => toast("To copied", "success"));
      };
    }

    if ($("rangeSummary")) {
      $("rangeSummary").textContent =
        s.range_summary ||
        (run ? "Scan in progress…" : "Scan stopped — auto-restarts if GPU free (no list scan)");
    }

    const mode = (s.mode || "").toLowerCase();
    if ($("rangeStartLabel")) {
      $("rangeStartLabel").textContent =
        mode === "sequential"
          ? "From (window start — full hex)"
          : "From (min threads — full hex)";
    }
    if ($("rangeEndLabel")) {
      $("rangeEndLabel").textContent =
        mode === "sequential"
          ? "To (window end — full hex)"
          : "To (max threads — full hex)";
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
        setPill("scanModePill", "STOPPED (auto GPU idle)", "warn");
        setText(
          "rangeHint",
          "No brute process — auto-restart ~15s if no GPU list scan"
        );
      } else if (mode === "random") {
        setPill("scanModePill", "RANDOM · live", "warn");
        setText(
          "rangeHint",
          "Random: From/To = min→max of last thread keys (samples, not a continuous range)"
        );
      } else if (mode === "sequential") {
        setPill("scanModePill", "SEQUENTIAL · live", "ok");
        const step = s.range_step || 1073741824;
        setText(
          "rangeHint",
          rs && re
            ? `Sequential: window From → To (${formatNumber(step)} keys/step). Live cursor below. Log = no retest.`
            : "Sequential: window start → window end (step 2^30 default)"
        );
      } else {
        setPill("scanModePill", "active", "ok");
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
    // Only override the toggle button if NO other scan source is running.
    // updateStatusBar (12s) aggregates brute+dict; we must not overwrite it.
    const dictRunning = !!(health?.dict?.running);
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
      setScanPill(
        "on",
        "SCAN ON",
        [
          ...hwParts,
          rate != null ? `${formatNumber(rate)} keys/s live` : null,
          s.keys_tested != null ? `${formatNumber(s.keys_tested)} tested` : null,
          majTip,
          rangeTip || null,
        ]
          .filter(Boolean)
          .join(" · ")
      );
    } else if (!dictRunning) {
      // Only set OFF if dict is also not running (updateStatusBar will handle the combined state)
      setScanPill("off", "SCAN OFF", "Background scan stopped");
    }
  }

  // ── GPU Temperature Display ────────────────────────────────────────────
  function updateGpuTemps(gpuRates) {
    const container = $("gpuTempPills");
    if (!container) return;
    if (!gpuRates.length) {
      container.innerHTML = "<span style='color:var(--text-dim)'>—</span>";
      return;
    }
    container.innerHTML = gpuRates
      .map((g) => {
        const temp = g.temperature_c != null ? Number(g.temperature_c).toFixed(0) : "—";
        const util = g.utilization_pct != null ? Number(g.utilization_pct).toFixed(0) : "—";
        const vram =
          g.vram_used_mb != null && g.vram_total_mb != null
            ? `${Math.round(g.vram_used_mb)} / ${Math.round(g.vram_total_mb)} MB`
            : "—";
        // Color based on temperature
        let cls = "ok";
        if (g.temperature_c != null) {
          if (g.temperature_c >= 80) cls = "hot";
          else if (g.temperature_c >= 70) cls = "warn";
        }
        return `<span class="gpu-temp-pill ${cls}" title="GPU ${g.id}: ${util}% util, ${vram} VRAM">
          GPU${g.id}: <strong>${temp}°C</strong> <span style="opacity:0.6">(${util}%)</span>
        </span>`;
      })
      .join("");
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
      setText("snapSize", s.size_mb != null ? s.size_mb.toFixed(0) + " MB" : "—");
      setText("indexLoaded", s.index_loaded ? "yes" : "no");
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
      // GPU monitoring
      try {
        if (h.gpu && Array.isArray(h.gpu)) updateGpuPanel(h.gpu);
      } catch (e) {
        console.warn("updateGpuPanel", e);
      }
      // Process monitor
      try {
        if (h.processes && Array.isArray(h.processes)) updateProcessPanel(h.processes);
      } catch (e) {
        console.warn("updateProcessPanel", e);
      }
      // Historical indexer status
      try {
        if (h.historical_indexer) updateHiStatus(h.historical_indexer);
      } catch (e) {
        console.warn("updateHiStatus", e);
      }
      // Scan optimization panel
      try {
        updateScanOptimization(h);
      } catch (e) {
        console.warn("updateScanOptimization", e);
      }
      // Success path always restores index pill if loaded
      if (h.index_loaded) {
        setPill(
          "pillReady",
          `Idx ${formatCompact(h.index_scripts || 0)}`,
          "ok",
          `Index loaded · ${formatNumber(h.index_scripts || 0)} scripts`
        );
      } else {
        setPill("pillReady", "Idx …", "warn", "Index loading");
      }
    } catch (e) {
      console.error("refreshHealth", e);
      setText("footerHealth", "API: " + (e.name === "AbortError" ? "timeout" : e.message || e));
      setPill("pillReady", "API …", "warn", e.message || "API unavailable");
    }
  }

  async function refreshScan() {
    try {
      const stats = await api("/api/scan/stats");
      updateScan(stats, window.__lastHealth || {});
      checkAlerts(stats, window.__lastHealth || {});
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
    btcAction("/api/bitcoind/restart", "Restart")
  );
  $("btnBtcStart")?.addEventListener("click", () => btcAction("/api/bitcoind/start", "Start"));
  $("btnBtcStop")?.addEventListener("click", () => btcAction("/api/bitcoind/stop", "Stop"));
  $("btnBtcRefresh")?.addEventListener("click", refreshBtc);
  $("btnBarRestart")?.addEventListener("click", () =>
    btcAction("/api/bitcoind/restart", "Restart")
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
        r.success ? `Index: ${formatNumber(r.index_scripts)} scripts` : "Index failed",
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
    if (!confirm("Rebuild UTXO (very long)?")) return;
    setMsg("snapMsg", "Rebuild…");
    try {
      await api("/api/snapshot/refresh", { method: "POST", body: "{}" });
      setMsg("snapMsg", "OK — reloading index", "success");
    } catch (e) {
      setMsg("snapMsg", e.message, "error");
    }
  });

  // Brute — CPU % ofs cores (défaut 50), configurable
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
          ? `→ ${workers} forced workers (fixed threads) · ${cores} cores detected`
          : p === 0
            ? `→ 0 CPU worker (GPU only) · ${cores} cores`
            : `→ ${workers} CPU workers (${p}% of ${cores} cores) — active par défaut`;
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
    $("rangeStepHint").textContent = `= ${expStr} = ${formatNumber(n)} keys`;
  }

  async function refreshRangesLog() {
    try {
      const log = await api("/api/scan/ranges");
      const ranges = Array.isArray(log.ranges) ? log.ranges : [];
      setText(
        "rangesDoneLabel",
        `${formatNumber(ranges.length)} range(s) logged`
      );
      if ($("rangesLogList")) {
        if (!ranges.length) {
          $("rangesLogList").textContent =
            "No range completed — log fills when a To is reached.";
        } else {
          // plus récentes en haut
          const lines = ranges
            .slice()
            .reverse()
            .slice(0, 40)
            .map((r, i) => {
              const n = ranges.length - i;
              return `#${n} From ${r.start}\n    To ${r.end}\n    ${formatNumber(r.keys || 0)} keys · ${r.completed_at || ""}`;
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
        // prefer current window for editing
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
      toast("Start hex required", "error");
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
          ? "Start applied — scan restarted"
          : "Start saved (next start)",
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
      toast(`Step saved: ${formatNumber(range_step)} keys`, "success");
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
    e.preventFromfault();
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
      toast(`Scan started · ${w} CPU workers`, "success");
    } catch (err) {
      setMsg("scanMsg", err.message, "error");
      toast(err.message, "error");
    }
  });
  $("btnStop")?.addEventListener("click", async () => {
    await api("/api/scan/stop", { method: "POST", body: "{}" });
    setMsg("scanMsg", "Stop requested");
  });

  // Scan toggle button — click to start/stop the brute scan
  let _scanTogglePending = false;
  $("pillScan")?.addEventListener("click", async () => {
    const btn = $("pillScan");
    if (!btn || _scanTogglePending) return;
    const currentState = btn.className || "";

    // If paused, resume
    if (window.__scanPaused && currentState.includes("is-off")) {
      if ($("btnScanResume")) $("btnScanResume").click();
      return;
    }

    if (currentState.includes("is-on")) {
      // Currently ON → stop
      _scanTogglePending = true;
      btn.style.pointerEvents = "none";
      try {
        await api("/api/scan/stop", { method: "POST", body: "{}" });
        setMsg("scanMsg", "Stop requested via toggle");
        toast("Scan stopped", "");
      } catch (err) {
        setMsg("scanMsg", "Stop error: " + err.message, "error");
        toast(err.message, "error");
      } finally {
        btn.style.pointerEvents = "";
        _scanTogglePending = false;
      }
    } else {
      // Currently OFF or ERROR → start
      _scanTogglePending = true;
      btn.style.pointerEvents = "none";
      setScanPill("off", "STARTING…", "Starting scan…", "");
      try {
        const transforms = selectedTransforms();
        renderActiveTransforms(transforms);
        const body = buildScanConfigBody();
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
        toast(`Scan started · ${w} CPU workers`, "success");
        // Auto-clear error in scan tile
        const errEl = $("infoScanError");
        if (errEl) errEl.textContent = "";
      } catch (err) {
        setScanPill("error", "SCAN FAILED", "Error: " + err.message, err.message);
        setMsg("scanMsg", "Failed to start: " + err.message, "error");
        toast("Scan failed to start: " + err.message, "error");
        // Auto-clear error state after 15s so user can retry
        setTimeout(() => {
          const btn2 = $("pillScan");
          if (btn2 && btn2.className?.includes("is-error")) {
            setScanPill("off", "SCAN OFF", "Click to start", "");
          }
        }, 15000);
      } finally {
        btn.style.pointerEvents = "";
        _scanTogglePending = false;
      }
    }
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
          if (msg.scan) updateScan(msg.scan, window.__lastHealth || {});
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
  initBip39Tab();
  refreshBtc();
  refreshSnap();
  refreshHealth();
  refreshScan();
  setInterval(refreshHealth, 12000);
  setInterval(refreshScan, 3000);
  refreshRangesLog();
  setInterval(refreshRangesLog, 15000);

  // Watchlist buttons
  $("btnAddWatchlist")?.addEventListener("click", showAddWatchlistDialog);
  $("btnExportWatchlist")?.addEventListener("click", () => {
    const list = loadWatchlist();
    if (!list.length) return toast("Watchlist vide", "warning");
    const csv = "key,source,added_at,last_balance_sats\n" + list.map(w => `${w.key},${w.source},${w.added_at},${w.last_balance}`).join("\n");
    downloadFile(`btcsolver-watchlist-${Date.now()}.csv`, csv, "text/csv");
    toast("Watchlist exported", "success");
  });
  $("btnClearWatchlist")?.addEventListener("click", () => {
    if (confirm("Vider la watchlist?")) {
      saveWatchlist([]);
      renderWatchlist();
      toast("Watchlist vidée", "");
    }
  });

  // Throughput chart — real-time keys/sec over last 30 minutes (5s intervals)
  (function initThroughputChart() {
    const canvas = $("throughputChart");
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    // HiDPI support
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    const W = rect.width;
    const H = rect.height;

    const MAX_POINTS = 360; // 30 min at 1 sample/5s
    const data = { total: [], gpu0: [], gpu1: [], gpu2: [], cpu: [] };
    const COLORS = {
      total: "#f7931a",
      gpu0: "#58a6ff",
      gpu1: "#3dd68c",
      gpu2: "#d29922",
      cpu: "#8b9bb0",
    };

    function pushPoint(stats) {
      if (!stats || !stats.running) return;
      const total = (stats.keys_per_sec_live || stats.keys_per_sec || 0) / 1e6;
      data.total.push(total);
      if (data.total.length > MAX_POINTS) data.total.shift();

      const gpuRates = stats.gpu_rates || [];
      for (let i = 0; i < 3; i++) {
        const arr = [data.gpu0, data.gpu1, data.gpu2][i];
        const g = gpuRates.find((r) => r.id === i);
        arr.push(g ? (g.keys_per_sec / 1e6) : 0);
        if (arr.length > MAX_POINTS) arr.shift();
      }
      const cpu = (stats.cpu_keys_per_sec || 0) / 1e6;
      data.cpu.push(cpu);
      if (data.cpu.length > MAX_POINTS) data.cpu.shift();
    }

    function drawChart() {
      ctx.clearRect(0, 0, W, H);
      if (data.total.length < 2) {
        ctx.fillStyle = "#5c6b7f";
        ctx.font = "12px Inter, sans-serif";
        ctx.textAlign = "center";
        ctx.fillText("Waiting for scan data…", W / 2, H / 2);
        return;
      }

      const allValues = [...data.total, ...data.gpu0, ...data.gpu1, ...data.gpu2];
      const maxVal = Math.max(...allValues, 1);
      const yRange = maxVal * 1.15;
      const pad = { top: 8, right: 12, bottom: 22, left: 52 };
      const chartW = W - pad.left - pad.right;
      const chartH = H - pad.top - pad.bottom;

      // Grid lines
      ctx.strokeStyle = "rgba(36, 48, 65, 0.6)";
      ctx.lineWidth = 0.5;
      const gridLines = 4;
      for (let i = 0; i <= gridLines; i++) {
        const y = pad.top + (chartH / gridLines) * i;
        ctx.beginPath();
        ctx.moveTo(pad.left, y);
        ctx.lineTo(W - pad.right, y);
        ctx.stroke();
        // Y-axis labels
        const val = yRange - (yRange / gridLines) * i;
        ctx.fillStyle = "#5c6b7f";
        ctx.font = "10px JetBrains Mono, monospace";
        ctx.textAlign = "right";
        ctx.fillText(val.toFixed(0) + "M", pad.left - 6, y + 3);
      }

      // X-axis time labels (30 min span, 5s intervals)
      ctx.fillStyle = "#5c6b7f";
      ctx.font = "9px JetBrains Mono, monospace";
      ctx.textAlign = "center";
      const timeLabels = [
        { pos: 0, label: "now" },
        { pos: MAX_POINTS / 6, label: "-5m" },
        { pos: MAX_POINTS / 3, label: "-10m" },
        { pos: MAX_POINTS / 2, label: "-15m" },
        { pos: (2 * MAX_POINTS) / 3, label: "-20m" },
        { pos: (5 * MAX_POINTS) / 6, label: "-25m" },
        { pos: MAX_POINTS - 1, label: "-30m" },
      ];
      for (const tl of timeLabels) {
        const x = pad.left + (tl.pos / (MAX_POINTS - 1)) * chartW;
        ctx.fillText(tl.label, x, H - 4);
      }

      // Draw lines
      function drawLine(values, color, lineWidth) {
        if (values.length < 2) return;
        const len = values.length;
        ctx.strokeStyle = color;
        ctx.lineWidth = lineWidth || 1.5;
        ctx.beginPath();
        for (let i = 0; i < len; i++) {
          const x = pad.left + (i / (MAX_POINTS - 1)) * chartW;
          const y = pad.top + chartH - (values[i] / yRange) * chartH;
          if (i === 0) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        }
        ctx.stroke();

        // Fill area under line (subtle)
        ctx.globalAlpha = 0.08;
        ctx.fillStyle = color;
        ctx.lineTo(pad.left + ((len - 1) / (MAX_POINTS - 1)) * chartW, pad.top + chartH);
        ctx.lineTo(pad.left, pad.top + chartH);
        ctx.closePath();
        ctx.fill();
        ctx.globalAlpha = 1.0;
      }

      // Draw from back to front
      drawLine(data.cpu, COLORS.cpu, 1);
      drawLine(data.gpu2, COLORS.gpu2, 1.2);
      drawLine(data.gpu1, COLORS.gpu1, 1.2);
      drawLine(data.gpu0, COLORS.gpu0, 1.2);
      drawLine(data.total, COLORS.total, 2);

      // Current value labels (right side)
      ctx.font = "10px JetBrains Mono, monospace";
      ctx.textAlign = "right";
      const labels = [
        { val: data.total, color: COLORS.total, prefix: "Σ " },
        { val: data.gpu0, color: COLORS.gpu0, prefix: "G0 " },
        { val: data.gpu1, color: COLORS.gpu1, prefix: "G1 " },
        { val: data.gpu2, color: COLORS.gpu2, prefix: "G2 " },
      ];
      labels.forEach((l, idx) => {
        if (l.val.length > 0) {
          const last = l.val[l.val.length - 1];
          const y = pad.top + 12 + idx * 13;
          ctx.fillStyle = l.color;
          ctx.fillText(l.prefix + last.toFixed(1) + "M/s", W - 4, y);
        }
      });
    }

    // Collect data every 5 seconds (independent of refreshScan)
    setInterval(async () => {
      try {
        const stats = await api("/api/scan/stats");
        pushPoint(stats);
      } catch (_) {}
      drawChart();
    }, 5000);

    // Initial draw
    drawChart();
  })();
  // Scan listes: 2 mises à jour / seconde
  setInterval(async () => {
    try {
      updateDict(await api("/api/dict/status"));
    } catch (_) {}
  }, 500);

  // ── Historical Archive Viewer ──────────────────────────────────────────
  function downloadFile(filename, content, mime) {
    const blob = new Blob([content], { type: mime || "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url; a.download = filename;
    document.body.appendChild(a); a.click();
    document.body.removeChild(a); URL.revokeObjectURL(url);
  }

  let archiveData = null;

  async function loadArchive() {
    try {
      const data = await api("/api/keys/archive");
      archiveData = data.keys || data.entries || data.archive || [];
      $("pillArchiveCount").textContent = `Total: ${archiveData.length} keys`;
      renderArchive();
    } catch (e) {
      $("archiveList").innerHTML = `<span class="hint">Failed to load archive: ${esc(String(e))}</span>`;
    }
  }

  function renderArchive() {
    if (!archiveData) return;
    const search = ($("archiveSearch")?.value || "").toLowerCase();
    const sort = $("archiveSort")?.value || "peak_desc";
    const spentOnly = $("archiveSpentOnly")?.checked || false;

    let filtered = archiveData;
    if (spentOnly) {
      filtered = filtered.filter(k => (k.current_sats || k.balance_sats || 0) === 0);
    }
    if (search) {
      filtered = filtered.filter(k => {
        const haystack = [
          k.key_hex, k.privkey_hex, k.address, k.addresses,
          k.peak_sats, k.current_sats, k.first_seen_height, k.last_seen_height,
          k.note, k.transform
        ].join(" ").toLowerCase();
        return haystack.includes(search);
      });
    }

    // Sort
    filtered.sort((a, b) => {
      switch (sort) {
        case "peak_desc": return (b.peak_sats || 0) - (a.peak_sats || 0);
        case "peak_asc": return (a.peak_sats || 0) - (b.peak_sats || 0);
        case "first_desc": return (b.first_seen_height || 0) - (a.first_seen_height || 0);
        case "last_desc": return (b.last_seen_height || 0) - (a.last_seen_height || 0);
        case "current_desc": return (b.current_sats || b.balance_sats || 0) - (a.current_sats || a.balance_sats || 0);
        default: return 0;
      }
    });

    const container = $("archiveList");
    if (!filtered.length) {
      container.innerHTML = '<span class="hint">No archived keys match your filters.</span>';
      return;
    }

    // Show top 200 by default to avoid DOM overload
    const maxShow = 200;
    const shown = filtered.slice(0, maxShow);

    let html = `<div style="font-size:0.8rem;opacity:0.7;margin-bottom:0.5rem">Showing ${shown.length} of ${filtered.length} archived keys</div>`;
    html += '<div style="display:grid;gap:0.5rem">';

    for (const k of shown) {
      const keyHex = k.key_hex || k.privkey_hex || "unknown";
      const shortKey = shortDisplay(keyHex, 8, 6);
      const peak = k.peak_sats || k.max_sats || 0;
      const current = k.current_sats ?? k.balance_sats ?? 0;
      const firstH = k.first_seen_height ?? k.first_height ?? "—";
      const lastH = k.last_seen_height ?? k.last_height ?? "—";
      const addr = k.address || (k.addresses && k.addresses[0]) || "—";
      const transform = k.transform || k.variant || "";
      const isSpent = current === 0;

      html += `<div class="match-item" style="${isSpent ? 'opacity:0.6' : ''}">
        <div class="match-header">
          <span class="match-key mono">${shortDisplay(keyHex, 12, 8)}</span>
          ${isSpent ? '<span class="badge badge-spent">SPENT</span>' : '<span class="badge badge-active">ACTIVE</span>'}
          ${transform ? `<span class="badge" style="background:var(--blue-dim);color:var(--blue)">${esc(transform)}</span>` : ''}
        </div>
        <div class="match-details">
          <span class="match-balance ${isSpent ? '' : 'balance-positive'}">${formatNumber(current)} sats</span>
          <span class="match-detail">Peak: <strong>${formatNumber(peak)} sats</strong></span>
          <span class="match-detail">First: ${formatHeight(firstH)}</span>
          <span class="match-detail">Last: ${formatHeight(lastH)}</span>
          <span class="match-detail mono" style="word-break:break-all">${shortDisplay(String(addr), 10, 8)}</span>
        </div>
      </div>`;
    }

    html += '</div>';
    if (filtered.length > maxShow) {
      html += `<div class="hint" style="text-align:center;margin-top:0.5rem">Use search/filter to narrow down (${filtered.length - maxShow} more hidden)</div>`;
    }
    container.innerHTML = html;
    wireCopyButtons(container);
  }

  // Archive event listeners
  if ($("btnRefreshArchive")) {
    $("btnRefreshArchive").addEventListener("click", loadArchive);
  }
  if ($("archiveSearch")) {
    $("archiveSearch").addEventListener("input", () => renderArchive());
  }
  if ($("archiveSort")) {
    $("archiveSort").addEventListener("change", () => renderArchive());
  }
  if ($("archiveSpentOnly")) {
    $("archiveSpentOnly").addEventListener("change", () => renderArchive());
  }
  if ($("btnExportArchiveCsv")) {
    $("btnExportArchiveCsv").addEventListener("click", () => {
      if (!archiveData || !archiveData.length) { toast("No archive data to export"); return; }
      let csv = "key_hex,address,peak_sats,current_sats,first_seen_height,last_seen_height,transform,status\n";
      for (const k of archiveData) {
        const keyHex = k.key_hex || k.privkey_hex || "";
        const addr = k.address || (k.addresses && k.addresses[0]) || "";
        const peak = k.peak_sats || k.max_sats || 0;
        const current = k.current_sats ?? k.balance_sats ?? 0;
        const firstH = k.first_seen_height ?? "";
        const lastH = k.last_seen_height ?? "";
        const transform = k.transform || k.variant || "";
        const status = current === 0 ? "spent" : "active";
        csv += `"${keyHex}","${addr}",${peak},${current},${firstH},${lastH},"${transform}",${status}\n`;
      }
      downloadFile("btc-archive.csv", csv, "text/csv");
      toast("Archive exported as CSV");
    });
  }

  // Load archive on startup
  void loadArchive();
  // Refresh archive every 60 seconds
  setInterval(loadArchive, 60000);

  // ── Alert Configuration ────────────────────────────────────────────────
  function loadAlertSettings() {
    try {
      const saved = localStorage.getItem("btcsolver_alert_settings_v1");
      if (saved) return JSON.parse(saved);
    } catch (_) {}
    return { gpuThreshold: 20, utxoThreshold: 24, soundEnabled: true, matchEnabled: true };
  }
  function saveAlertSettings(s) {
    try { localStorage.setItem("btcsolver_alert_settings_v1", JSON.stringify(s)); } catch (_) {}
  }
  (function initAlertConfig() {
    const settings = loadAlertSettings();
    const gpuSlider = $("alertGpuThreshold");
    const gpuVal = $("alertGpuThresholdVal");
    const utxoSlider = $("alertUtxoThreshold");
    const utxoVal = $("alertUtxoThresholdVal");
    const soundCheck = $("alertSoundEnabled");
    const matchCheck = $("alertMatchEnabled");
    const resetBtn = $("btnResetAlerts");
    if (!gpuSlider || !utxoSlider) return;

    // Load saved settings
    gpuSlider.value = settings.gpuThreshold;
    gpuVal.textContent = settings.gpuThreshold + "%";
    utxoSlider.value = settings.utxoThreshold;
    utxoVal.textContent = settings.utxoThreshold + "h";
    soundCheck.checked = settings.soundEnabled;
    matchCheck.checked = settings.matchEnabled;

    // Update on change
    const save = () => {
      const s = {
        gpuThreshold: parseInt(gpuSlider.value),
        utxoThreshold: parseInt(utxoSlider.value),
        soundEnabled: soundCheck.checked,
        matchEnabled: matchCheck.checked
      };
      gpuVal.textContent = s.gpuThreshold + "%";
      utxoVal.textContent = s.utxoThreshold + "h";
      saveAlertSettings(s);
      // Also update alertConfig for the existing alert system
      alertConfig.gpu_drop_threshold = s.gpuThreshold;
      alertConfig.utxo_stale_hours = s.utxoThreshold;
      saveAlertConfig();
    };
    gpuSlider.addEventListener("input", save);
    utxoSlider.addEventListener("input", save);
    soundCheck.addEventListener("change", save);
    matchCheck.addEventListener("change", save);
    if (resetBtn) {
      resetBtn.addEventListener("click", () => {
        gpuSlider.value = 20; utxoSlider.value = 24;
        soundCheck.checked = true; matchCheck.checked = true;
        save();
        toast("Alert settings reset to defaults");
      });
    }
    // Initialize alertConfig with loaded settings
    alertConfig.gpu_drop_threshold = settings.gpuThreshold;
    alertConfig.utxo_stale_hours = settings.utxoThreshold;
  })();

  // ── Scan Progress Visualization ────────────────────────────────────────
  (function initScanProgress() {
    const updateProgress = (stats) => {
      if (!stats) return;
      const keysEl = $("scanProgressKeys");
      const rangesEl = $("scanProgressRanges");
      const rangeEl = $("scanProgressRange");
      const coverageEl = $("scanProgressCoverage");
      const barEl = $("scanProgressBar");
      const labelEl = $("scanProgressLabel");
      if (!keysEl) return;

      const keys = stats.keys_tested || 0;
      const ranges = stats.ranges_done || 0;
      const rangeStep = stats.range_step || 0;
      const rangeStart = stats.range_start || "";
      const rangeEnd = stats.range_end || "";

      keysEl.textContent = formatNumber(keys);
      rangesEl.textContent = ranges;

      // Current range display
      if (rangeStart && rangeEnd) {
        const shortStart = rangeStart.slice(0, 16) + "...";
        const shortEnd = rangeEnd.slice(0, 16) + "...";
        rangeEl.textContent = `${shortStart} → ${shortEnd}`;
      }

      // Progress within current range
      let pct = 0;
      if (rangeStep > 0 && keys > 0) {
        // Estimate: keys tested within current range / range step
        const totalRangeKeys = ranges * rangeStep + (keys % rangeStep);
        const currentRangeProgress = keys % rangeStep;
        pct = rangeStep > 0 ? (currentRangeProgress / rangeStep) * 100 : 0;
      }
      pct = Math.min(100, Math.max(0, pct));
      if (barEl) barEl.style.width = pct + "%";
      if (labelEl) labelEl.textContent = pct.toFixed(2) + "%";

      // Key space coverage (for the scanned ranges)
      if (coverageEl) {
        // Total keys scanned vs 2^256 (impossible to meaningfully express)
        // Instead, show the number of ranges and keys in a meaningful way
        const totalScanned = keys;
        const rangesCovered = ranges;
        coverageEl.textContent = `${rangesCovered} ranges (${formatNumber(totalScanned)} keys)`;
      }
    };

    // Update progress every 5 seconds from scan stats
    setInterval(async () => {
      try {
        const stats = await api("/api/scan/stats");
        updateProgress(stats);
      } catch (_) {}
    }, 5000);

    // Initial update
    setTimeout(async () => {
      try { updateProgress(await api("/api/scan/stats")); } catch (_) {}
    }, 2000);
  })();

  // === GPU Monitoring Panel ===
  function updateGpuPanel(gpus) {
    const container = $("gpuCards");
    const timeEl = $("gpuRefreshTime");
    if (!container) return;
    if (timeEl) timeEl.textContent = `Updated ${new Date().toLocaleTimeString()}`;
    let html = '';
    for (const g of gpus) {
      const util = g.util_pct ?? 0;
      const memUsed = g.mem_used_mb ?? 0;
      const memTotal = g.mem_total_mb ?? 0;
      const temp = g.temp_c ?? 0;
      const power = g.power_w ?? 0;
      const powerLimit = g.power_limit ?? 0;
      const memPct = memTotal > 0 ? Math.round(memUsed / memTotal * 100) : 0;
      const utilColor = util > 60 ? 'var(--green)' : util > 20 ? 'var(--accent)' : 'var(--red)';
      const tempColor = temp > 80 ? 'var(--red)' : temp > 70 ? 'var(--warning)' : 'var(--green)';
      html += `<div style="border:1px solid var(--border);border-radius:8px;padding:1rem;background:var(--card-bg)">
        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem">
          <strong>GPU ${g.index} — ${g.name || 'Unknown'}</strong>
          <span style="font-size:0.8rem;color:var(--text-muted)">${power.toFixed(0)}W${powerLimit > 0 ? '/' + powerLimit.toFixed(0) + 'W' : ''}</span>
        </div>
        <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.5rem;font-size:0.8rem">
          <div>
            <div style="color:var(--text-muted);margin-bottom:0.2rem">Utilization</div>
            <div style="font-size:1.2rem;font-weight:700;color:${utilColor}" class="mono">${util}%</div>
            <div class="progress-bar" style="height:6px;margin-top:0.2rem"><div class="progress-fill" style="width:${util}%;background:${utilColor}"></div></div>
          </div>
          <div>
            <div style="color:var(--text-muted);margin-bottom:0.2rem">VRAM</div>
            <div style="font-size:1.2rem;font-weight:700" class="mono">${memUsed} / ${memTotal} GB</div>
            <div class="progress-bar" style="height:6px;margin-top:0.2rem"><div class="progress-fill" style="width:${memPct}%;background:var(--accent)"></div></div>
          </div>
          <div>
            <div style="color:var(--text-muted);margin-bottom:0.2rem">Temperature</div>
            <div style="font-size:1.2rem;font-weight:700;color:${tempColor}" class="mono">${temp}°C</div>
          </div>
          <div>
            <div style="color:var(--text-muted);margin-bottom:0.2rem">VRAM Usage</div>
            <div style="font-size:1.2rem;font-weight:700" class="mono">${memPct}%</div>
          </div>
        </div>
      </div>`;
    }
    container.innerHTML = html;
  }

  // === Process Monitor Panel ===
  function updateProcessPanel(procs) {
    const container = $("processList");
    if (!container) return;
    let html = '';
    for (const p of procs) {
      const status = p.running ? '✅' : '⬛';
      const color = p.running ? 'var(--green)' : 'var(--text-muted)';
      const mem = p.mem_mb ? `${(p.mem_mb / 1024).toFixed(1)} GB` : '';
      const pid = p.pid ? `PID ${p.pid}` : '';
      html += `<div style="border:1px solid var(--border);border-radius:6px;padding:0.75rem;background:var(--card-bg);text-align:center">
        <div style="font-size:1.3rem;margin-bottom:0.3rem">${status}</div>
        <div style="font-weight:600;font-size:0.85rem;color:${color}">${p.name}</div>
        <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.2rem" class="mono">${pid} ${mem}</div>
      </div>`;
    }
    container.innerHTML = html;
  }

  // === Historical Indexer Status ===
  function updateHiStatus(hi) {
    const runningEl = $("hiRunning");
    const progressEl = $("hiProgress");
    const outputEl = $("hiOutput");
    const badgeEl = $("hiBadge");
    const badgeTextEl = $("hiBadgeText");
    if (!runningEl) return;
    if (hi.running) {
      runningEl.textContent = '🔄 Running';
      runningEl.style.color = 'var(--green)';
      if (badgeEl) badgeEl.className = 'status-badge';
      if (badgeTextEl) badgeTextEl.textContent = 'Active';
    } else {
      runningEl.textContent = '⬛ Stopped';
      runningEl.style.color = 'var(--text-muted)';
      if (badgeEl) badgeEl.className = 'status-badge';
      if (badgeTextEl) badgeTextEl.textContent = 'Idle';
    }
    if (hi.checkpoint) {
      progressEl.textContent = hi.checkpoint.substring(0, 80) + (hi.checkpoint.length > 80 ? '...' : '');
    } else {
      progressEl.textContent = 'No checkpoint';
    }
    // Check output file size
    const outputFiles = ['data/historical-scripts.bin', 'data/historical-scripts.bin.tmp'];
    outputEl.textContent = outputFiles.join(', ');
  }

  // === Strategies Tab: Key Hunter Intelligence ===
  const STRATEGIES_DATA = [
    {
      title: "🔥 Mots de passe simples (2009-2011)",
      prob: "Très élevée",
      probColor: "var(--green)",
      desc: "Les premiers utilisateurs de Bitcoin utilisaient souvent des mots de passe triviaux: 'password', 'bitcoin', '123456', leur nom, leur email.",
      transforms: ["SHA256", "double SHA256", "MD5 padded"],
      tips: ["Testez les noms communs: john, mike, sarah", "Emails complets: user@gmail.com", "Dates: 1985, 20090103"]
    },
    {
      title: "📖 Citations & proverbes",
      prob: "Élevée",
      probColor: "#3dd68c",
      desc: "Beaucoup de brainwallets utilisaient des citations de films, livres, ou proverbes. En anglais surtout (communauté Bitcoin anglophone).",
      transforms: ["SHA256", "lowercase", "no spaces"],
      tips: ["Movie quotes: 'May the force be with you'", "Biblical verses: 'In the beginning'", "Proverbs: 'A penny saved is a penny earned'"]
    },
    {
      title: "🎵 Paroles de chansons populaires",
      prob: "Élevée",
      probColor: "#3dd68c",
      desc: "Paroles de chansons des années 60-90 étaient très populaires comme brainwallets. Les utilisateurs prenaient leur refrain préféré.",
      transforms: ["SHA256", "lowercase", "strip symbols"],
      tips: ["Beatles, Queen, Led Zeppelin, Nirvana", "Refrains: 'I want to hold your hand'", "Avec années: 'bohemian rhapsody 1975'"]
    },
    {
      title: "🔢 Constantes mathématiques",
      prob: "Moyenne",
      probColor: "var(--accent)",
      desc: "π, e, √2, φ (nombre d'or) — tronqués ou décalés. Les geeks aimaient ces constantes comme source d'entropie.",
      transforms: ["256 bits de π[offset]", "e[offset]", "√n pour n=2..1000"],
      tips: ["π à partir du 100ème chiffre", "e × 1000 mod n", "Combinaisons: π + e concaténés"]
    },
    {
      title: "🎮 Culture geek & gaming (2009)",
      prob: "Moyenne-élevée",
      probColor: "var(--accent)",
      desc: "World of Warcraft, chess openings, D&D, Magic: The Gathering — la communauté Bitcoin 2009 était très geek.",
      transforms: ["SHA256", "lowercase", "suffix year"],
      tips: ["WoW character names + server", "Chess: 'e4 e5 Nf3'", "MTG card names: 'Black Lotus'"]
    },
    {
      title: "🌍 Noms propres + suffixes",
      prob: "Moyenne",
      probColor: "var(--accent)",
      desc: "Prénom + nom de famille, nom de chien/chat, ville natale — combinés avec des suffixes courants.",
      transforms: ["SHA256", "prefix 'my'", "suffix 'bitcoin', 'wallet', '2009'"],
      tips: ["john smith bitcoin", "my dog rex wallet", "paris france btc", "avec années: 1985, 1990, 2009"]
    },
    {
      title: "🃏 Séquences de cartes / dés / pièces",
      prob: "Basse-moyenne",
      probColor: "#d29922",
      desc: "Séquences de jets de dés (1-6), tirages de cartes (A-K), ou piles de pièces (H/T) converties en hex.",
      transforms: ["binaire → hex 256 bits", "D6 → 2.5 bits par jet"],
      tips: ["100 jets de D6 → 166 bits", "52 cartes mélangées → 225 bits", "Piles: HHTHTT... → bits"]
    },
    {
      title: "📱 Numéros de téléphone / cartes",
      prob: "Basse",
      probColor: "#d29922",
      desc: "Numéros de téléphone, numéros de carte de crédit (sécursés), codes postaux — utilisés comme seed.",
      transforms: ["SHA256 du numéro", "padded à 256 bits"],
      tips: ["Téléphone US: 5551234567", "Code postal + date naissance", "Numéro étudiante / employé"]
    },
    {
      title: "🧮 Opérations mathématiques simples",
      prob: "Basse-moyenne",
      probColor: "#d29922",
      desc: "n², n³, 2^n, n! — résultats convertis en hex 256 bits. Les mathématiciens aimaient cette approche.",
      transforms: ["n² mod 2^256", "2^n en hex", "factorielle tronquée"],
      tips: ["123456789² en hex", "2^127 - 1 (Mersenne)", "Fibonacci[100] en hex"]
    },
    {
      title: "🌐 URLs & domaines (2009)",
      prob: "Basse",
      probColor: "#d29922",
      desc: "URLs de sites populaires 2009: forums Bitcoin précoces, blogs crypto, sites de gambling.",
      transforms: ["SHA256 de l'URL", "sans http://", "domaine seulement"],
      tips: ["bitcointalk.org", "bitcoin.org", "bitcoinxt.org", "names.bitcoin.nu"]
    },
    {
      title: "🎹 Patterns de clavier",
      prob: "Moyenne",
      probColor: "var(--accent)",
      desc: "Patterns visuels sur le clavier: zigzag, cercle, ligne. Très communs comme mots de passe faibles.",
      transforms: ["SHA256 du pattern", "qwerty", "1234567890"],
      tips: ["qwertyuiop", "asdfghjkl", "zxcvbnm", "1qaz2wsx", "pattern en Z"]
    },
    {
      title: "📅 Dates significatives",
      prob: "Élevée",
      probColor: "#3dd68c",
      desc: "Dates de naissance, anniversaires, dates historiques — formatées de différentes manières.",
      transforms: ["SHA256", "YYYYMMDD", "DD-MM-YYYY", "avec texte"],
      tips: ["20090103 (genesis block)", "19760509 (Satoshi?)", "01/01/2009", "january 3 2009"]
    },
  ];

  // ============================================================
  // BIP39 Tab
  // ============================================================

  const BIP39_WORD_COUNT = 2048;
  const BIP39_SCAN_RATE = 180_000_000; // keys/sec

  const BIP39_PATTERNS = [
    { name: "All same word", desc: "abandon abandon abandon ...", example: "abandon × 12", difficulty: "2048 (1 word to guess)" },
    { name: "2-word pattern", desc: "Only 2 distinct words repeated", example: "word1 word2 word1 word2 ...", difficulty: "2048² = 4.2M" },
    { name: "First/last known", desc: "First and last words remembered", example: "abandon ??? ??? ??? ??? abandon", difficulty: "2048⁴ = 1.7T" },
    { name: "Common words only", desc: "Everyday vocabulary (≈500 words)", example: "house cat tree mountain ...", difficulty: "500¹² ≈ 2.4×10³²" },
    { name: "Alphabetical order", desc: "Words in dictionary order", example: "abandon ability able about ...", difficulty: "Very constrained" },
    { name: "Date-based", desc: "Birth dates, anniversaries as words", example: "january three two zero nine", difficulty: "Limited set" },
    { name: "Names", desc: "Pet names, family names, celebrities", example: "lucky dog john mary ...", difficulty: "~1000 common names" },
    { name: "Keyboard patterns", desc: "QWERTY patterns converted to words", example: "qwerty → closest BIP39 words", difficulty: "Very limited" },
    { name: "Partial phrase (1 unknown)", desc: "11/12 words known", example: "11 known + 1 unknown", difficulty: "2048 — feasible!" },
    { name: "Partial phrase (2 unknown)", desc: "10/12 words known", example: "10 known + 2 unknown", difficulty: "4.2M — feasible with GPU" },
    { name: "Partial phrase (3 unknown)", desc: "9/12 words known", example: "9 known + 3 unknown", difficulty: "8.6B — needs time" },
    { name: "Repeated groups", desc: "Groups of words repeated", example: "abc abc abc abc", difficulty: "2048³ = 8.6B" },
  ];

  function initBip39Tab() {
    // Input handler
    const input = $("bip39Input");
    if (input) {
      input.addEventListener("input", updateBip39Calc);
      input.addEventListener("keydown", (e) => {
        if (e.key === "Enter" && !e.shiftKey) {
          e.preventDefault();
          testBip39Phrase();
        }
      });
    }

    // Buttons
    $("btnBip39Test")?.addEventListener("click", testBip39Phrase);
    $("btnBip39Batch")?.addEventListener("click", batchScanBip39);
    $("btnBip39Clear")?.addEventListener("click", clearBip39);
    $("btnBip39Export")?.addEventListener("click", exportBip39Results);

    // Render patterns
    renderBip39Patterns();
  }

  function updateBip39Calc() {
    const input = $("bip39Input");
    if (!input) return;

    const text = input.value.trim().toLowerCase();
    const words = text ? text.split(/\s+/) : [];
    const total = words.length;
    const unknown = words.filter(w => w === "?" || w === "_" || w === "???").length;
    const known = total - unknown;

    // Update stats
    setText("bip39TotalWords", total);
    setText("bip39KnownWords", known);
    setText("bip39UnknownWords", unknown);

    // Calculate combinations
    if (unknown === 0) {
      setText("bip39Combinations", total > 0 ? "1 (complete)" : "—");
      setText("bip39EstTime", "—");
    } else if (unknown <= 6) {
      const combos = Math.pow(BIP39_WORD_COUNT, unknown);
      setText("bip39Combinations", formatScientific(combos));
      // Estimate scan time
      const seconds = combos / BIP39_SCAN_RATE;
      setText("bip39EstTime", formatBip39Duration(seconds));
    } else {
      setText("bip39Combinations", formatScientific(Math.pow(BIP39_WORD_COUNT, unknown)));
      setText("bip39EstTime", "∞ (impractical)");
    }

    // Validity check
    const validityEl = $("bip39Validity");
    if (total === 0) {
      setText("bip39Validity", "—");
      validityEl.style.color = "";
    } else if ([12, 15, 18, 21, 24].includes(total)) {
      setText("bip39Validity", "✓ Valid length");
      validityEl.style.color = "var(--green)";
    } else {
      setText("bip39Validity", "⚠ Invalid length (need 12/15/18/21/24)");
      validityEl.style.color = "var(--yellow)";
    }

    // Word-by-word analysis
    renderBip39WordAnalysis(words);
  }

  function renderBip39WordAnalysis(words) {
    const container = $("bip39WordAnalysis");
    if (!container) return;

    if (words.length === 0) {
      container.innerHTML = "";
      return;
    }

    let html = '<div style="display:flex;flex-wrap:wrap;gap:0.3rem;margin-top:0.3rem">';
    words.forEach((w, i) => {
      const isUnknown = w === "?" || w === "_" || w === "???";
      const isValid = !isUnknown && w.length >= 3; // rough check
      const cls = isUnknown ? "background:var(--red-dim);color:var(--red);border-color:rgba(248,81,73,0.3)"
        : isValid ? "background:var(--green-dim);color:var(--green);border-color:rgba(61,214,140,0.3)"
        : "background:var(--yellow-dim);color:var(--yellow);border-color:rgba(210,153,34,0.3)";
      html += `<span style="display:inline-block;padding:0.15rem 0.4rem;border-radius:4px;font-size:0.72rem;font-family:var(--mono);border:1px solid;${cls}">${i + 1}. ${w}</span>`;
    });
    html += '</div>';
    container.innerHTML = html;
  }

  function renderBip39Patterns() {
    const container = $("bip39Patterns");
    if (!container) return;

    container.innerHTML = BIP39_PATTERNS.map((p, i) => `
      <div class="strategy-card" style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--bg-elevated);cursor:pointer"
           onclick="document.getElementById('bip39Input').value='${p.example.replace(/'/g, "\\'")}';updateBip39Calc()">
        <strong style="font-size:0.85rem">${p.name}</strong>
        <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.2rem">${p.desc}</div>
        <div style="font-size:0.7rem;color:var(--accent);margin-top:0.3rem">Difficulty: ${p.difficulty}</div>
      </div>
    `).join("");
  }

  async function testBip39Phrase() {
    const input = $("bip39Input");
    if (!input || !input.value.trim()) {
      toast("Enter a BIP39 phrase", "error");
      return;
    }

    const text = input.value.trim();
    const words = text.split(/\s+/).filter(w => w !== "?" && w !== "_" && w !== "???");
    const unknown = text.split(/\s+/).filter(w => w === "?" || w === "_" || w === "???").length;

    if (unknown > 0) {
      const combos = Math.pow(BIP39_WORD_COUNT, unknown);
      if (combos > 1_000_000_000) {
        toast(`Too many combinations (${formatScientific(combos)}) — use Batch scan with GPU`, "error");
        return;
      }
    }

    // For complete phrases, test directly
    if (unknown === 0) {
      try {
        const r = await api("/api/keys/check", {
          method: "POST",
          body: JSON.stringify({ key: text, format: "bip39" })
        });
        displayBip39Results([r]);
        toast(`Tested: ${r.sats > 0 ? "BALANCE FOUND!" : "No balance"}`, r.sats > 0 ? "success" : "");
      } catch (e) {
        toast(`Error: ${e.message}`, "error");
      }
    } else {
      // For partial phrases, suggest batch scan
      toast(`${formatScientific(Math.pow(BIP39_WORD_COUNT, unknown))} combinations — use Batch scan`, "");
    }
  }

  async function batchScanBip39() {
    const input = $("bip39Input");
    if (!input || !input.value.trim()) {
      toast("Enter a BIP39 phrase pattern", "error");
      return;
    }

    const text = input.value.trim();
    const words = text.split(/\s+/);
    const unknown = words.filter(w => w === "?" || w === "_" || w === "???").length;

    if (unknown === 0) {
      toast("Phrase is complete — use Test button instead", "");
      return;
    }

    if (unknown > 3) {
      toast(`⚠ ${formatScientific(Math.pow(BIP39_WORD_COUNT, unknown))} combinations — this will take a very long time`, "error");
      return;
    }

    // Start the scan via the easy-keys endpoint or dict scan
    // For now, generate a corpus file and start scanning
    toast(`Starting BIP39 batch scan: ${formatScientific(Math.pow(BIP39_WORD_COUNT, unknown))} combinations`, "");

    // Use the dict scan with BIP39 transform
    try {
      await api("/api/dict/start", {
        method: "POST",
        body: JSON.stringify({
          phrases: text,
          sha256: false,
          double: false,
          md5: false,
          revChars: false,
          revWords: false,
          lower: true,
          stripSym: false,
          noSpace: false,
          upper: false,
          bip39: true,
          bip39_all_paths: true,
          bip39_address_count: 10
        })
      });
      toast("BIP39 batch scan started", "success");
    } catch (e) {
      toast(`Error: ${e.message}`, "error");
    }
  }

  function clearBip39() {
    $("bip39Input").value = "";
    updateBip39Calc();
    $("bip39ResultsCard").hidden = true;
  }

  function displayBip39Results(results) {
    const card = $("bip39ResultsCard");
    const container = $("bip39Results");
    if (!card || !container) return;

    card.hidden = false;
    const hasBalance = results.some(r => r.sats > 0);

    container.innerHTML = results.map(r => `
      <div class="match-item" style="${r.sats > 0 ? 'border-color:var(--green);background:var(--green-dim)' : ''}">
        <div class="match-header">
          <span class="type">${r.sats > 0 ? '🟢 HIT' : '⚪ No balance'}</span>
          <span class="mono" style="font-size:0.75rem">${(r.addresses || []).join(", ") || "—"}</span>
        </div>
        ${r.sats > 0 ? `
          <div class="match-details">
            <span class="match-balance balance-positive">${formatSats(r.sats)}</span>
            <span>${(r.sats / 100_000_000).toFixed(8)} BTC</span>
          </div>
        ` : ''}
      </div>
    `).join("");
  }

  function exportBip39Results() {
    toast("Export not yet implemented", "");
  }

  // Helper: format number in scientific notation
  function formatScientific(n) {
    if (n < 1000) return n.toString();
    if (n < 1_000_000) return `${(n / 1000).toFixed(0)}K`;
    if (n < 1_000_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n < 1_000_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
    if (n < 1_000_000_000_000_000) return `${(n / 1_000_000_000_000).toFixed(1)}T`;
    // Scientific notation for huge numbers
    const exp = Math.floor(Math.log10(n));
    const mantissa = (n / Math.pow(10, exp)).toFixed(1);
    return `${mantissa}×10${toSuperscript(exp)}`;
  }

  function toSuperscript(n) {
    const sup = { '0': '⁰', '1': '¹', '2': '²', '3': '³', '4': '⁴', '5': '⁵', '6': '⁶', '7': '⁷', '8': '⁸', '9': '⁹' };
    return String(n).split('').map(c => sup[c] || c).join('');
  }

  function formatBip39Duration(seconds) {
    if (seconds < 60) return `${Math.round(seconds)}s`;
    if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
    if (seconds < 86400) return `${Math.round(seconds / 3600)}h`;
    if (seconds < 31536000) return `${Math.round(seconds / 86400)}d`;
    return `${(seconds / 31536000).toFixed(1)} years`;
  }

  function renderStrategies() {
    const container = $("strategyCards");
    if (!container) return;
    const grid = container.querySelector('.strategies-grid') || container;
    container.innerHTML = STRATEGIES_DATA.map((s, i) => `
      <div class="strategy-card" style="border:1px solid var(--border);border-radius:8px;padding:1rem;background:var(--card-bg);cursor:pointer" data-strategy="${i}">
        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem">
          <strong style="font-size:0.9rem">${s.title}</strong>
          <span style="font-size:0.75rem;padding:0.2rem 0.5rem;border-radius:4px;background:${s.probColor}22;color:${s.probColor}">Prob: ${s.prob}</span>
        </div>
        <p style="font-size:0.8rem;color:var(--text-muted);margin:0.3rem 0">${s.desc}</p>
        <div style="font-size:0.75rem;margin-top:0.5rem">
          <div style="color:var(--text-dim);margin-bottom:0.2rem">Transforms: ${s.transforms.join(', ')}</div>
          <div style="color:var(--accent);margin-top:0.3rem">
            ${s.tips.map(t => `<div>• ${t}</div>`).join('')}
          </div>
        </div>
      </div>
    `).join('');
  }

  // === Pattern Analysis ===
  function renderPatternAnalysis() {
    const container = $("patternAnalysis");
    if (!container) return;
    const patterns = [
      { pattern: "SHA256(mot_simple)", coverage: "~50M corpus", scanned: "✅", priority: 1 },
      { pattern: "SHA256(citation_film)", coverage: "~100K phrases", scanned: "✅", priority: 2 },
      { pattern: "SHA256(parole_chanson)", coverage: "~200K phrases", scanned: "✅", priority: 2 },
      { pattern: "SHA256(prenom + nom)", coverage: "~1M combos", scanned: "⏳", priority: 3 },
      { pattern: "π/e/√n → 256 bits", coverage: "~10K variants", scanned: "✅", priority: 3 },
      { pattern: "SHA256(date YYYYMMDD)", coverage: "~30K dates", scanned: "✅", priority: 4 },
      { pattern: "SHA256(URL 2009)", coverage: "~5K URLs", scanned: "⏳", priority: 4 },
      { pattern: "Keyboard patterns", coverage: "~10K patterns", scanned: "✅", priority: 3 },
      { pattern: "Dés/cartes/pieces → hex", coverage: "~1M combos", scanned: "⏳", priority: 5 },
      { pattern: "n²/2^n/n! → hex 256b", coverage: "~100K values", scanned: "✅", priority: 4 },
      { pattern: "BIP39 phrases courantes", coverage: "~500K phrases", scanned: "✅", priority: 2 },
      { pattern: "WIF faible (entropie < 64 bits)", coverage: "brute-force", scanned: "🔄", priority: 1 },
    ];
    const sorted = patterns.sort((a, b) => a.priority - b.priority);
    container.innerHTML = `
      <table style="width:100%;border-collapse:collapse;font-size:0.8rem">
        <thead><tr style="border-bottom:1px solid var(--border);text-align:left">
          <th style="padding:0.5rem">Priorité</th>
          <th>Pattern</th>
          <th>Couverture</th>
          <th>Statut</th>
          <th>Action</th>
        </tr></thead>
        <tbody>
          ${sorted.map((p, i) => `
            <tr style="border-bottom:1px solid var(--border);opacity:${1 - i * 0.05}">
              <td style="padding:0.4rem 0.5rem;font-weight:700;color:${p.priority <= 2 ? 'var(--green)' : p.priority <= 3 ? 'var(--accent)' : 'var(--text-muted)'}">${p.priority}</td>
              <td><code>${p.pattern}</code></td>
              <td>${p.coverage}</td>
              <td>${p.scanned}</td>
              <td><button class="btn btn-ghost btn-sm scan-pattern-btn" data-pattern="${i}">Scan</button></td>
            </tr>
          `).join('')}
        </tbody>
      </table>
      <p class="hint" style="margin-top:0.5rem;font-size:0.75rem">
        💡 Les patterns marqués ✅ ont été scannés via les corpus existants. 🔄 = scan GPU en continu. ⏳ = à planifier.
      </p>
    `;
  }

  // === Watchlist System ===
  const WATCHLIST_KEY = "btcsolver_watchlist_v1";
  function loadWatchlist() {
    try { return JSON.parse(localStorage.getItem(WATCHLIST_KEY) || "[]"); } catch { return []; }
  }
  function saveWatchlist(list) {
    localStorage.setItem(WATCHLIST_KEY, JSON.stringify(list.slice(0, 200)));
  }

  function renderWatchlist() {
    const container = $("watchlistTable");
    const countEl = $("pillWatchlistCount");
    if (!container) return;
    const list = loadWatchlist();
    if (countEl) countEl.textContent = list.length;
    if (!list.length) {
      container.innerHTML = '<p class="hint">Watchlist vide. Ajoutez des clés hex (64 chars) pour les surveiller.</p>';
      return;
    }
    container.innerHTML = `
      <table style="width:100%;border-collapse:collapse;font-size:0.75rem">
        <thead><tr style="border-bottom:1px solid var(--border);text-align:left;position:sticky;top:0;background:var(--card-bg)">
          <th style="padding:0.4rem">Clé (hex)</th>
          <th>Adresse</th>
          <th>Source</th>
          <th>Ajoutée le</th>
          <th>Statut</th>
          <th>Action</th>
        </tr></thead>
        <tbody>
          ${list.map((w, i) => `
            <tr style="border-bottom:1px solid var(--border)">
              <td style="padding:0.3rem;font-family:var(--mono);font-size:0.7rem">${shortDisplay(w.key, 8, 6)}</td>
              <td style="font-family:var(--mono);font-size:0.7rem">${w.address ? shortDisplay(w.address, 8, 6) : '—'}</td>
              <td>${esc(w.source || 'manual')}</td>
              <td>${w.added_at ? new Date(w.added_at).toLocaleDateString() : '—'}</td>
              <td>${w.last_balance > 0 ? '<span style="color:var(--green)">💰 ' + w.last_balance + ' sats</span>' : w.checked ? '<span style="color:var(--text-muted)">✓ vérifiée</span>' : '<span style="color:var(--text-muted)">⏳</span>'}</td>
              <td><button class="btn btn-ghost btn-sm" data-wl-rm="${i}" style="padding:0.1rem 0.3rem;font-size:0.7rem">✕</button></td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    `;
    container.querySelectorAll("[data-wl-rm]").forEach((b) => {
      b.addEventListener("click", () => {
        const list = loadWatchlist();
        list.splice(Number(b.dataset.wlRm), 1);
        saveWatchlist(list);
        renderWatchlist();
      });
    });
  }

  // === Brainwallet Patterns 2009 ===
  function renderBrainwalletPatterns() {
    const container = $("brainwalletPatterns");
    if (!container) return;
    const patterns = [
      { cat: "Constantes mathématiques", examples: [
        "π → 256 bits (décimales 1-32)",
        "e → 256 bits",
        "√2, √3, √5 → 256 bits",
        "φ (nombre d'or) → 256 bits",
        "ln(2), ln(10) → 256 bits",
        "γ (Euler-Mascheroni) → 256 bits",
      ]},
      { cat: "Opérations simples", examples: [
        "n² mod 2^256 pour n=1..10M",
        "2^n pour n=1..255",
        "n! (factorielle) tronqué",
        "Fibonacci[n] pour n=1..200",
        "Premiers nombres premiers concaténés",
        "n³, n⁴, n^5 → hex",
      ]},
      { cat: "Noms + années", examples: [
        "'bitcoin 2009', 'bitcoin 2010'",
        "'satoshi nakamoto 1975'",
        "'my bitcoin wallet 2009'",
        "'first bitcoin 2009'",
        "'genesis block 2009'",
        "'one bitcoin please'",
      ]},
      { cat: "Phrases en anglais", examples: [
        "'i love bitcoin'",
        "'to the moon'",
        "'hodl forever'",
        "'money of the people'",
        "'end inflation'",
        "'free money for all'",
      ]},
      { cat: "Patterns clavier", examples: [
        "'qwertyuiop' (ligne haute)",
        "'asdfghjkl' (ligne milieu)",
        "'1234567890' (chiffres)",
        "'1qaz2wsx3edc' (vertical)",
        "'zaq1xsw2' (zigzag)",
        "'!@#$%^&*()' (symboles)",
      ]},
      { cat: "Dates significatives", examples: [
        "'january 3 2009' (genesis)",
        "'may 22 2010' (pizza day)",
        "'december 6 2017' (all time high)",
        "Date naissance: 'march 15 1985'",
        "Anniversaire: '01/01/1990'",
        "Événements: '9/11', 'august 9 2009'",
      ]},
      { cat: "Citations cultes", examples: [
        "'May the force be with you'",
        "'I'll be back'",
        "'Here's looking at you kid'",
        "'Elementary my dear Watson'",
        "'To infinity and beyond'",
        "'After all tomorrow and tomorrow and tomorrow'",
      ]},
      { cat: "Math → hex créatif", examples: [
        "π × e → 256 bits",
        "√(π × e) → 256 bits",
        "2^256 - 1 (max key)",
        "2^127 - 1 (Mersenne prime)",
        "0xdeadbeaf... (joke hex)",
        "0xcafebabe... (joke hex)",
      ]},
    ];
    container.innerHTML = patterns.map(p => `
      <div style="margin-bottom:1.5rem">
        <strong style="color:var(--accent)">${p.cat}</strong>
        <ul style="margin:0.3rem 0;padding-left:1.2rem;font-size:0.8rem;color:var(--text-muted)">
          ${p.examples.map(e => `<li style="margin-bottom:0.2rem">${e}</li>`).join('')}
        </ul>
      </div>
    `).join('');
  }

  // === Quick Actions ===
  function renderQuickActions() {
    const container = $("quickActions");
    if (!container) return;
    const actions = [
      { label: "🔑 Scan Easy Keys", desc: "Lance le corpus merged (24.4M clés)", action: "easykeys" },
      { label: "📊 Export Stats", desc: "Export JSON des stats scan", action: "exportstats" },
      { label: "🔄 Reload Index", desc: "Recharge le FlatIndex en RAM", action: "reloadindex" },
      { label: "⏸ Pause Scan", desc: "Pause le scan GPU/CPU", action: "pause" },
      { label: "▶ Resume Scan", desc: "Reprendre le scan", action: "resume" },
      { label: "🛑 Stop Scan", desc: "Arrêter le scan", action: "stop" },
      { label: "📋 Export Archive", desc: "Export CSV des clés actives", action: "exportarchive" },
      { label: "🔍 Check Health", desc: "Refresh complet du système", action: "health" },
    ];
    container.innerHTML = actions.map(a => `
      <button class="btn btn-secondary btn-sm quick-action-btn" data-action="${a.action}" style="text-align:left;padding:0.75rem;font-size:0.8rem;white-space:normal;height:auto">
        <div style="font-weight:600">${a.label}</div>
        <div style="font-size:0.7rem;color:var(--text-muted);font-weight:400">${a.desc}</div>
      </button>
    `).join('');

    container.querySelectorAll(".quick-action-btn").forEach(btn => {
      btn.addEventListener("click", async () => {
        const action = btn.dataset.action;
        try {
          switch (action) {
            case "easykeys":
              btn.disabled = true;
              toast("Starting corpus scan…", "");
              await api("/api/scan/easy-keys", { method: "POST", body: JSON.stringify({ use_gpu: false }) });
              toast("Corpus scan launched", "success");
              break;
            case "exportstats":
              const res = await fetch("/api/scan/export");
              const blob = await res.blob();
              const a = document.createElement("a");
              a.href = URL.createObjectURL(blob);
              a.download = `btcsolver-stats-${Date.now()}.json`;
              a.click();
              toast("Stats exported", "success");
              break;
            case "reloadindex":
              toast("Reloading index…", "");
              await api("/api/index/reload", { method: "POST", body: "{}" });
              toast("Index reloaded", "success");
              break;
            case "pause":
              await api("/api/scan/pause", { method: "POST", body: "{}" });
              toast("Scan paused", "");
              break;
            case "resume":
              await api("/api/scan/resume", { method: "POST", body: "{}" });
              toast("Scan resumed", "success");
              break;
            case "stop":
              await api("/api/scan/stop", { method: "POST", body: "{}" });
              toast("Scan stopped", "error");
              break;
            case "exportarchive":
              const arcRes = await fetch("/api/keys/archive/export");
              const arcBlob = await arcRes.blob();
              const arcA = document.createElement("a");
              arcA.href = URL.createObjectURL(arcBlob);
              arcA.download = `btcsolver-archive-${Date.now()}.csv`;
              arcA.click();
              toast("Archive exported", "success");
              break;
            case "health":
              await refreshHealth();
              toast("Health refreshed", "success");
              break;
          }
        } catch (e) {
          toast("Action failed: " + e.message, "error");
        } finally {
          btn.disabled = false;
        }
      });
    });
  }

  // === UTXO1 Historical Index Status ===
  async function refreshUtxo1Status() {
    const statsEl = $("utxo1Stats");
    const pillEl = $("pillUtxo1Status");
    if (!statsEl) return;
    try {
      const data = await api("/api/utxo1/stats");
      if (data.exists) {
        if (pillEl) { pillEl.textContent = `✅ ${formatNumber(data.scripts)} scripts`; pillEl.className = "pill ok"; }
        statsEl.innerHTML = `
          <div style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--card-bg);text-align:center">
            <div style="font-size:0.75rem;color:var(--text-muted)">Scripts uniques</div>
            <div style="font-size:1.5rem;font-weight:700;color:var(--green)" class="mono">${formatNumber(data.scripts)}</div>
          </div>
          <div style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--card-bg);text-align:center">
            <div style="font-size:0.75rem;color:var(--text-muted)">Taille fichier</div>
            <div style="font-size:1.5rem;font-weight:700;color:var(--accent)" class="mono">${data.file_size_mb.toFixed(1)} MB</div>
          </div>
          <div style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--card-bg);text-align:center">
            <div style="font-size:0.75rem;color:var(--text-muted)">Version</div>
            <div style="font-size:1.5rem;font-weight:700" class="mono">v${data.version}</div>
          </div>
          <div style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--card-bg);text-align:center">
            <div style="font-size:0.75rem;color:var(--text-muted)">Format</div>
            <div style="font-size:1rem;font-weight:600" class="mono">${data.format}</div>
          </div>
        `;
      } else {
        if (pillEl) { pillEl.textContent = "⏳ En construction"; pillEl.className = "pill warn"; }
        const tmpGb = data.tmp_file_size_gb ? data.tmp_file_size_gb.toFixed(1) : '?';
        statsEl.innerHTML = `
          <div style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--card-bg);text-align:center;grid-column:1/-1">
            <div style="font-size:0.85rem;color:var(--text-muted)">Index en construction — fichier tmp: ${tmpGb} GB</div>
            <div style="font-size:0.75rem;color:var(--text-muted);margin-top:0.3rem">${data.message || 'Merge en cours...'}</div>
          </div>
        `;
      }
    } catch (e) {
      if (pillEl) pillEl.textContent = "Erreur";
      statsEl.innerHTML = `<p class="hint">Error: ${e.message}</p>`;
    }
  }

  // === UTXO1 Query ===
  async function queryUtxo1() {
    const input = $("utxo1QueryInput");
    const result = $("utxo1QueryResult");
    if (!input || !result) return;
    const script = input.value.trim();
    if (!script) { result.textContent = "Enter script hex"; result.style.color = "var(--warning)"; return; }
    result.textContent = "Querying..."; result.style.color = "var(--text-muted)";
    try {
      const data = await api("/api/utxo1/query", {
        method: "POST",
        body: JSON.stringify({ script }),
      });
      if (data.found) {
        result.textContent = "✅ FOUND — script was active!";
        result.style.color = "var(--green)";
        toast("Script found in historical index!", "success");
      } else {
        result.textContent = "✕ Not found — never active";
        result.style.color = "var(--text-muted)";
      }
    } catch (e) {
      result.textContent = "Error: " + e.message;
      result.style.color = "var(--red)";
    }
  }

  // === Performance History Tracker ===
  const PERF_HISTORY_KEY = "btcsolver_perf_history_v1";
  const perfHistoryData = {
    timestamps: [],
    throughput: [],
    gpuUtil: [],
    maxPoints: 200, // ~10 min at 3s intervals
  };

  function loadPerfHistory() {
    try {
      const saved = localStorage.getItem(PERF_HISTORY_KEY);
      if (saved) {
        const d = JSON.parse(saved);
        if (d.throughput) perfHistoryData.throughput = d.throughput;
        if (d.timestamps) perfHistoryData.timestamps = d.timestamps;
        if (d.gpuUtil) perfHistoryData.gpuUtil = d.gpuUtil;
      }
    } catch {}
  }

  function savePerfHistory() {
    try {
      localStorage.setItem(PERF_HISTORY_KEY, JSON.stringify({
        timestamps: perfHistoryData.timestamps,
        throughput: perfHistoryData.throughput,
        gpuUtil: perfHistoryData.gpuUtil,
      }));
    } catch {}
  }

  function updatePerfHistory(scanStats) {
    if (!scanStats) return;
    const now = Date.now();
    const kps = (scanStats.keys_per_sec || scanStats.keys_per_second || 0) / 1e6; // M k/s
    const gpuUtil = scanStats.gpu_util || 0;

    perfHistoryData.timestamps.push(now);
    perfHistoryData.throughput.push(kps);
    perfHistoryData.gpuUtil.push(gpuUtil);

    // Trim to max points
    while (perfHistoryData.timestamps.length > perfHistoryData.maxPoints) {
      perfHistoryData.timestamps.shift();
      perfHistoryData.throughput.shift();
      perfHistoryData.gpuUtil.shift();
    }

    savePerfHistory();
    renderPerfHistory();
    drawPerfHistoryChart();
  }

  function renderPerfHistory() {
    const container = $("perfHistory");
    if (!container) return;
    const tp = perfHistoryData.throughput;
    const gu = perfHistoryData.gpuUtil;
    const current = tp.length > 0 ? tp[tp.length - 1] : 0;
    const avg = tp.length > 0 ? tp.reduce((a, b) => a + b, 0) / tp.length : 0;
    const best = tp.length > 0 ? Math.max(...tp) : 0;
    const gpuAvg = gu.length > 0 ? gu.reduce((a, b) => a + b, 0) / gu.length : 0;

    container.innerHTML = `
      <div style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--card-bg);text-align:center">
        <div style="font-size:0.75rem;color:var(--text-muted)">Throughput actuel</div>
        <div style="font-size:1.3rem;font-weight:700;color:var(--accent)" class="mono">${current.toFixed(1)} M/s</div>
      </div>
      <div style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--card-bg);text-align:center">
        <div style="font-size:0.75rem;color:var(--text-muted)">Moyenne</div>
        <div style="font-size:1.3rem;font-weight:700;color:var(--green)" class="mono">${avg.toFixed(1)} M/s</div>
      </div>
      <div style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--card-bg);text-align:center">
        <div style="font-size:0.75rem;color:var(--text-muted)">Meilleur</div>
        <div style="font-size:1.3rem;font-weight:700;color:var(--green)" class="mono">${best.toFixed(1)} M/s</div>
      </div>
      <div style="padding:0.75rem;border:1px solid var(--border);border-radius:8px;background:var(--card-bg);text-align:center">
        <div style="font-size:0.75rem;color:var(--text-muted)">GPU avg</div>
        <div style="font-size:1.3rem;font-weight:700;color:${gpuAvg > 50 ? 'var(--green)' : 'var(--accent)'}" class="mono">${gpuAvg.toFixed(0)}%</div>
      </div>
    `;
  }

  function drawPerfHistoryChart() {
    const canvas = $("perfHistoryChart");
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    const W = rect.width;
    const H = rect.height;

    ctx.clearRect(0, 0, W, H);
    const tp = perfHistoryData.throughput;
    if (tp.length < 2) {
      ctx.fillStyle = "#5c6b7f";
      ctx.font = "12px Inter, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText("Collecting performance data...", W / 2, H / 2);
      return;
    }

    const maxVal = Math.max(...tp, 1) * 1.1;
    const pad = { top: 8, right: 12, bottom: 18, left: 48 };
    const chartW = W - pad.left - pad.right;
    const chartH = H - pad.top - pad.bottom;

    // Grid
    ctx.strokeStyle = "rgba(36, 48, 65, 0.6)";
    ctx.lineWidth = 0.5;
    for (let i = 0; i <= 3; i++) {
      const y = pad.top + (chartH / 3) * i;
      ctx.beginPath(); ctx.moveTo(pad.left, y); ctx.lineTo(W - pad.right, y); ctx.stroke();
      const val = maxVal - (maxVal / 3) * i;
      ctx.fillStyle = "#5c6b7f";
      ctx.font = "10px JetBrains Mono, monospace";
      ctx.textAlign = "right";
      ctx.fillText(val.toFixed(0) + "M", pad.left - 6, y + 3);
    }

    // Throughput line
    const len = tp.length;
    ctx.strokeStyle = "#f7931a";
    ctx.lineWidth = 2;
    ctx.beginPath();
    for (let i = 0; i < len; i++) {
      const x = pad.left + (i / (perfHistoryData.maxPoints - 1)) * chartW;
      const y = pad.top + chartH - (tp[i] / maxVal) * chartH;
      if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    }
    ctx.stroke();

    // Fill
    ctx.globalAlpha = 0.1;
    ctx.fillStyle = "#f7931a";
    ctx.lineTo(pad.left + ((len - 1) / (perfHistoryData.maxPoints - 1)) * chartW, pad.top + chartH);
    ctx.lineTo(pad.left, pad.top + chartH);
    ctx.closePath();
    ctx.fill();
    ctx.globalAlpha = 1.0;

    // Current value label
    ctx.fillStyle = "#f7931a";
    ctx.font = "bold 11px JetBrains Mono, monospace";
    ctx.textAlign = "right";
    ctx.fillText("▸ " + tp[len - 1].toFixed(1) + "M/s", W - 4, pad.top + 12);
  }

  // === Watchlist Add Dialog ===
  function showAddWatchlistDialog() {
    const key = prompt("Entrez la clé privée hex (64 chars) à ajouter à la watchlist:");
    if (!key || !/^[0-9a-fA-F]{64}$/.test(key.trim())) {
      if (key) toast("Clé invalide (doit être 64 chars hex)", "error");
      return;
    }
    const source = prompt("Source (optionnel):") || "manual";
    const list = loadWatchlist();
    // Dedup
    if (list.some(w => w.key.toLowerCase() === key.trim().toLowerCase())) {
      toast("Clé déjà dans la watchlist", "warning");
      return;
    }
    list.push({
      key: key.trim().toLowerCase(),
      source,
      added_at: new Date().toISOString(),
      last_balance: 0,
      checked: false,
    });
    saveWatchlist(list);
    renderWatchlist();
    toast("Clé ajoutée à la watchlist", "success");
  }

  // === Scan Optimization Panel ===
  function updateScanOptimization(h) {
    const throughputEl = $("optThroughput");
    const trendEl = $("optThroughputTrend");
    const gpuEffEl = $("optGpuEfficiency");
    const gpuDetailEl = $("optGpuDetail");
    const scoreEl = $("optScore");
    const scoreLabelEl = $("optScoreLabel");
    const recEl = $("optRecommendations");
    const timeEl = $("optRefreshTime");
    if (!throughputEl) return;
    if (timeEl) timeEl.textContent = `Updated ${new Date().toLocaleTimeString()}`;

    // Calculate throughput from scan stats
    const scan = h.scan || {};
    const keysPerSec = scan.keys_per_second || scan.keysPerSecond || 0;
    const totalKeys = scan.keys_tested || scan.keysTested || 0;
    const isRunning = scan.running === true;

    // Throughput display
    if (keysPerSec > 0) {
      if (keysPerSec > 1e9) throughputEl.textContent = (keysPerSec / 1e9).toFixed(1) + 'B k/s';
      else if (keysPerSec > 1e6) throughputEl.textContent = (keysPerSec / 1e6).toFixed(1) + 'M k/s';
      else if (keysPerSec > 1e3) throughputEl.textContent = (keysPerSec / 1e3).toFixed(1) + 'K k/s';
      else throughputEl.textContent = keysPerSec.toFixed(0) + ' k/s';
      throughputEl.style.color = keysPerSec > 100e6 ? 'var(--green)' : keysPerSec > 10e6 ? 'var(--accent)' : 'var(--red)';
    } else {
      throughputEl.textContent = isRunning ? 'Calcul...' : 'Arrêté';
      throughputEl.style.color = isRunning ? 'var(--text-muted)' : 'var(--red)';
    }

    // Trend
    const prevKeys = window.__prevScanKeys || 0;
    const prevTime = window.__prevScanTime || 0;
    if (totalKeys > 0 && prevTime > 0) {
      const dt = (Date.now() - prevTime) / 1000;
      if (dt > 0) {
        const rate = (totalKeys - prevKeys) / dt;
        const prevRate = window.__prevKeysPerSec || 0;
        if (prevRate > 0) {
          const pctChange = ((rate - prevRate) / prevRate * 100).toFixed(0);
          if (pctChange > 5) { trendEl.textContent = '📈 +' + pctChange + '%'; trendEl.style.color = 'var(--green)'; }
          else if (pctChange < -5) { trendEl.textContent = '📉 ' + pctChange + '%'; trendEl.style.color = 'var(--red)'; }
          else { trendEl.textContent = '➡️ stable'; trendEl.style.color = 'var(--text-muted)'; }
        } else { trendEl.textContent = '📊 baseline'; trendEl.style.color = 'var(--text-muted)'; }
        window.__prevKeysPerSec = rate;
      }
    }
    window.__prevScanKeys = totalKeys;
    window.__prevScanTime = Date.now();

    // GPU efficiency
    const gpus = h.gpu || [];
    let avgUtil = 0;
    let gpuDetails = [];
    if (gpus.length > 0) {
      for (const g of gpus) {
        const util = g.util_pct || 0;
        avgUtil += util;
        gpuDetails.push(`GPU${g.index}:${util}%`);
      }
      avgUtil = Math.round(avgUtil / gpus.length);
    }
    gpuEffEl.textContent = avgUtil + '%';
    gpuEffEl.style.color = avgUtil > 60 ? 'var(--green)' : avgUtil > 30 ? 'var(--accent)' : 'var(--red)';
    gpuDetailEl.textContent = gpuDetails.join(' ');

    // Performance score (0-100)
    let score = 0;
    let recs = [];
    // Throughput score (0-40 points)
    if (keysPerSec > 500e6) score += 40;
    else if (keysPerSec > 200e6) score += 35;
    else if (keysPerSec > 100e6) score += 30;
    else if (keysPerSec > 50e6) score += 20;
    else if (keysPerSec > 10e6) score += 10;
    // GPU utilization score (0-30 points)
    if (avgUtil > 80) score += 30;
    else if (avgUtil > 60) score += 25;
    else if (avgUtil > 40) score += 20;
    else if (avgUtil > 20) score += 10;
    // GPU count score (0-15 points)
    if (gpus.length >= 3) score += 15;
    else if (gpus.length >= 2) score += 10;
    else if (gpus.length >= 1) score += 5;
    // Running bonus (0-15 points)
    if (isRunning) score += 15;

    scoreEl.textContent = score + '/100';
    scoreEl.style.color = score >= 80 ? 'var(--green)' : score >= 50 ? 'var(--accent)' : 'var(--red)';
    if (score >= 80) scoreLabelEl.textContent = 'Excellent';
    else if (score >= 60) scoreLabelEl.textContent = 'Bon';
    else if (score >= 40) scoreLabelEl.textContent = 'Moyen';
    else scoreLabelEl.textContent = 'À améliorer';

    // Recommendations
    if (!isRunning) {
      recs.push('⚠️ <strong>Scan arrêté</strong> — lancez brute_force pour commencer le scan');
    }
    if (avgUtil < 20 && isRunning) {
      recs.push('🔴 <strong>GPU sous-utilisé (' + avgUtil + '%)</strong> — vérifiez llama-server qui concurrence les GPU');
      recs.push('💡 <strong>Action:</strong> Redémarrez llama-server avec <code>--ngl 0</code> pour libérer les GPU');
    }
    if (avgUtil < 60 && avgUtil >= 20 && isRunning) {
      recs.push('🟡 <strong>GPU partiellement utilisé (' + avgUtil + '%)</strong> — llama-server occupe partiellement les GPU');
      recs.push('💡 <strong>Action:</strong> Isoler llama-server sur un GPU dédié ou réduire <code>--tensor-split</code>');
    }
    if (keysPerSec < 50e6 && keysPerSec > 0) {
      recs.push('🐌 <strong>Throughput faible</strong> — vérifiez que le FlatIndex est chargé en VRAM (mode FULL)');
    }
    if (gpus.length === 0) {
      recs.push('📺 <strong>Aucun GPU détecté</strong> — vérifiez les drivers NVIDIA et libsecp_gpu.dll');
    }
    const utxoLag = (h.core_utxo || {}).utxo_lag_hours || 0;
    if (utxoLag > 24) {
      recs.push('🕐 <strong>UTXO obsolète (' + utxoLag + 'h)</strong> — rebuild nécessaire pour des résultats fiables');
    } else if (utxoLag > 0) {
      recs.push('✅ UTXO à jour (' + utxoLag + 'h de retard — acceptable)');
    }
    const hi = h.historical_indexer || {};
    if (hi.running) {
      recs.push('📊 <strong>Historical indexer actif</strong> — construction utxo1 en cours (consomme RAM CPU)');
    } else {
      recs.push('📊 Historical indexer inactif — lancez pour indexer toutes les clés ayant eu de l\'activité');
    }
    if (recs.length === 0) {
      recs.push('✅ Tout fonctionne optimalement — scan en cours, GPU bien utilisés');
    }
    recEl.innerHTML = recs.map(r => '<div style="margin-bottom:0.3rem">' + r + '</div>').join('');
  }

  // === Per-device Scan Breakdown (vertical GPU/CPU rows) ===
  let scanConfigCache = null;
  let scanConfigCacheTime = 0;

  async function getScanConfig() {
    const now = Date.now();
    if (scanConfigCache && now - scanConfigCacheTime < 10000) return scanConfigCache;
    try {
      scanConfigCache = await api("/api/scan/config");
      scanConfigCacheTime = now;
      return scanConfigCache;
    } catch { return scanConfigCache || {}; }
  }

  async function renderScanDeviceBreakdown(scanData, dictData, cpuTotal) {
    const container = $("scanDeviceBreakdown");
    if (!container) return;

    const config = await getScanConfig();
    const gpusStr = config.gpus || "0,1,2";
    const gpuIds = gpusStr.split(',').map(s => parseInt(s.trim())).filter(n => !isNaN(n));
    const cpuThreads = config.resolved_cpu_threads || config.threads || 0;
    const useGpu = config.use_gpu !== false;

    // Build GPU rows from scanData.gpu_rates
    const gpuRates = Array.isArray(scanData.gpu_rates) ? scanData.gpu_rates : [];
    const rows = [];

    // GPU rows
    for (const g of gpuRates) {
      const gpuId = g.id ?? 0;
      const rate = Number(g.keys_per_sec || g.keys_per_sec_avg || 0);
      const isActive = gpuIds.includes(gpuId) && useGpu;
      rows.push({
        type: 'gpu',
        id: gpuId,
        label: `GPU ${gpuId}`,
        icon: isActive ? '🟢' : '⚫',
        rate,
        threads: 1, // 1 host thread per GPU
        enabled: isActive,
      });
    }

    // Add inactive GPUs from config
    for (const gid of gpuIds) {
      if (!gpuRates.find(g => (g.id ?? 0) === gid) && useGpu) {
        rows.push({
          type: 'gpu', id: gid, label: `GPU ${gid}`, icon: '⚪',
          rate: 0, threads: 1, enabled: true,
        });
      }
    }

    // CPU row
    rows.push({
      type: 'cpu',
      id: 'cpu',
      label: 'CPU',
      icon: cpuThreads > 0 ? '🟢' : '⚫',
      rate: cpuTotal,
      threads: cpuThreads,
      enabled: cpuThreads > 0,
    });

    container.innerHTML = rows.map(r => {
      const rateStr = r.rate > 0 ? formatCompact(r.rate) + '/s' : '—';
      const opacity = r.enabled ? '1' : '0.4';
      const threadVal = r.type === 'cpu' ? r.threads : (r.enabled ? 1 : 0);
      const disabledAttr = !r.enabled ? 'disabled' : '';
      return `
        <div class="scan-device-row" style="display:flex;align-items:center;gap:0.5rem;padding:0.2rem 0.4rem;border-radius:6px;background:rgba(255,255,255,0.03);opacity:${opacity};cursor:pointer" onclick="toggleScanDevice('${r.type === 'cpu' ? 'cpu' : 'gpu' + r.id}')" title="Click to ${r.enabled ? 'disable' : 'enable'} ${r.label}">
          <span style="font-size:0.9rem;min-width:20px;text-align:center">${r.icon}</span>
          <span style="font-size:0.78rem;font-weight:600;min-width:55px;color:var(--text)">${r.label}</span>
          <span style="font-size:0.7rem;color:var(--text-muted);flex:1;text-align:right">${rateStr}</span>
          <input type="number" class="mono" min="0" max="128" value="${threadVal}" style="width:48px;font-size:0.72rem;padding:2px 4px;background:var(--bg-input);border:1px solid var(--border);border-radius:4px;color:var(--text);text-align:center" onclick="event.stopPropagation()" onchange="updateDeviceThreads('${r.type === 'cpu' ? 'cpu' : 'gpu' + r.id}', this.value)" title="${r.type === 'cpu' ? 'CPU threads (0=off)' : 'GPU threads (0=off, 1=on)'}">
        </div>`;
    }).join('');
  }

  // Global functions for onclick handlers
  window.toggleScanDevice = async function(device) {
    try {
      const config = await api("/api/scan/config");
      const currentThreads = device === 'cpu' ? (config.threads || 0) : 1;
      const newThreads = currentThreads > 0 ? 0 : (device === 'cpu' ? 16 : 1);
      const resp = await api("/api/scan/toggle-device", {
        method: "POST",
        body: JSON.stringify({ device, threads: newThreads }),
      });
      toast(`${device.toUpperCase()} ${newThreads === 0 ? 'disabled' : 'enabled'}${resp.scan_restarted ? ' — scan restarted' : ''}`, newThreads === 0 ? 'warning' : 'success');
    } catch (e) {
      toast("Toggle error: " + e.message, "error");
    }
  };

  window.updateDeviceThreads = async function(device, value) {
    const threads = parseInt(value) || 0;
    try {
      const resp = await api("/api/scan/toggle-device", {
        method: "POST",
        body: JSON.stringify({ device, threads }),
      });
      toast(`${device.toUpperCase()} → ${threads} thread${threads > 1 ? 's' : ''}${resp.scan_restarted ? ' (scan restarted)' : ''}`, 'success');
    } catch (e) {
      toast("Error: " + e.message, "error");
    }
  };

  // === Performance Benchmark ===
  let benchPollInterval = null;

  function initBenchmarkTab() {
    // Load current config
    loadBenchmarkConfig();

    // Buttons
    $("btnBenchRun")?.addEventListener("click", runBenchmark);
    $("btnBenchReset")?.addEventListener("click", resetBenchmark);
    $("btnBenchApplyBest")?.addEventListener("click", applyBestBenchmark);

    // Auto-poll benchmark status
    benchPollInterval = setInterval(pollBenchmarkStatus, 3000);
  }

  async function loadBenchmarkConfig() {
    try {
      const data = await api("/api/scan/config");
      if ($("benchLogicalCores")) $("benchLogicalCores").textContent = data.logical_cores || "—";
      // Compute GPU count from gpus string ("0,1,2" → 3) or default 3
      const gpuIds = data.gpus ? data.gpus.split(',').filter(s => s.trim()) : [];
      const gpuCount = gpuIds.length > 0 ? gpuIds.length : 3;
      if ($("benchGpuCount")) $("benchGpuCount").textContent = gpuCount;
      if ($("benchCpuThreads")) $("benchCpuThreads").textContent = data.resolved_cpu_threads ?? data.threads ?? "—";
      if ($("benchGpuBatch")) {
        // batch_size from config is in keys, convert to M
        const batchSize = data.batch_size || 33554432;
        $("benchGpuBatch").textContent = Math.round(batchSize / 1_000_000) + "M";
      }
    } catch {}
  }

  async function runBenchmark() {
    const cpuThreadsStr = $("benchCpuThreadsInput")?.value || "0,2,4,8,12,16,24,32";
    const cpuThreads = cpuThreadsStr.split(',').map(s => parseInt(s.trim())).filter(n => !isNaN(n) && n >= 0);
    const gpuBatchM = parseInt($("benchGpuBatchInput")?.value) || 32;
    const durationSecs = parseInt($("benchDurationInput")?.value) || 15;

    if (cpuThreads.length === 0) {
      toast("Enter at least one CPU thread count", "warning");
      return;
    }

    try {
      const resp = await api("/api/benchmark/run", {
        method: "POST",
        body: JSON.stringify({ cpu_threads: cpuThreads, gpu_batch_m: gpuBatchM, duration_secs: durationSecs }),
      });
      window.__benchRunning = true;
      toast(resp.message || "Benchmark started", "success");
      $("benchProgress").style.display = "block";
      $("benchResults").style.display = "none";
    } catch (e) {
      toast("Benchmark error: " + e.message, "error");
    }
  }

  async function pollBenchmarkStatus() {
    try {
      const data = await api("/api/benchmark/status");
      const progress = data.progress;
      if (!progress) return;

      if (progress.phase === "testing") {
        $("benchProgress").style.display = "block";
        $("benchProgressMsg").textContent = progress.message || "Testing...";
        const pct = progress.total > 0 ? Math.round((progress.current / progress.total) * 100) : 0;
        $("benchProgressCount").textContent = `${progress.current}/${progress.total}`;
        if ($("benchProgressBar")) $("benchProgressBar").style.width = pct + "%";
      } else if (progress.phase === "complete") {
        window.__benchRunning = false;
        $("benchProgress").style.display = "none";
        $("benchResults").style.display = "block";
        renderBenchmarkResults(data.results || []);
        toast("Benchmark complete!", "success");
      } else if (progress.phase === "error") {
        window.__benchRunning = false;
        $("benchProgress").style.display = "none";
        toast("Benchmark error: " + (progress.error || "unknown"), "error");
      }
    } catch {}
  }

  function renderBenchmarkResults(results) {
    const tbody = $("benchResultsBody");
    if (!tbody) return;

    const best = results.reduce((a, b) => a.keys_per_sec > b.keys_per_sec ? a : b, results[0]);

    tbody.innerHTML = results.map((r, i) => {
      const isBest = r === best;
      const rowStyle = isBest ? 'background:rgba(34,197,94,0.08);font-weight:600' : '';
      const kps = formatKps(r.keys_per_sec);
      const kpsGpu = formatKps(Math.round(r.keys_per_sec_per_gpu));
      const total = formatNum(r.total_keys);
      return `<tr style="${rowStyle};border-bottom:1px solid var(--border)">
        <td style="padding:0.5rem">${isBest ? '🏆 ' : ''}${r.label}</td>
        <td style="padding:0.5rem" class="mono">${r.cpu_threads}</td>
        <td style="padding:0.5rem" class="mono">${r.gpu_batch_m}M</td>
        <td style="padding:0.5rem" class="mono" style="color:${isBest ? 'var(--green)' : 'var(--text)'}">${kps}</td>
        <td style="padding:0.5rem" class="mono">${kpsGpu}</td>
        <td style="padding:0.5rem" class="mono">${total}</td>
        <td style="padding:0.5rem">${isBest ? '<span class="pill" style="background:var(--green);color:#000">BEST</span>' : ''}</td>
      </tr>`;
    }).join('');

    // Show best config
    if (best) {
      const bestEl = $("benchBestConfig");
      const bestText = $("benchBestText");
      if (bestEl && bestText) {
        bestEl.style.display = "block";
        bestText.textContent = `${best.label} → ${formatKps(best.keys_per_sec)} keys/sec (${formatKps(Math.round(best.keys_per_sec_per_gpu))}/GPU)`;
      }
    }
  }

  async function resetBenchmark() {
    try {
      window.__benchRunning = false;
      await api("/api/benchmark/reset", { method: "POST" });
      $("benchResults").style.display = "none";
      $("benchProgress").style.display = "none";
      toast("Benchmark results cleared", "");
    } catch (e) {
      toast("Reset error: " + e.message, "error");
    }
  }

  async function applyBestBenchmark() {
    try {
      const data = await api("/api/benchmark/status");
      const best = data.best;
      if (!best) {
        toast("No benchmark results yet", "warning");
        return;
      }
      // Get current config first, then update threads + batch_size
      const current = await api("/api/scan/config");
      const updated = {
        ...current,
        threads: best.cpu_threads,
        batch_size: best.gpu_batch_m * 1_000_000,
      };
      await api("/api/scan/config", {
        method: "POST",
        body: JSON.stringify(updated),
      });
      toast(`Applied: ${best.label} (${formatKps(best.keys_per_sec)})`, "success");
      loadBenchmarkConfig();
    } catch (e) {
      toast("Apply error: " + e.message, "error");
    }
  }

  function formatKps(n) {
    if (n >= 1_000_000_000) return (n / 1_000_000_000).toFixed(1) + "B";
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + "M";
    if (n >= 1_000) return (n / 1_000).toFixed(1) + "K";
    return n.toString();
  }

  function formatNum(n) {
    if (n == null || n === "" || Number.isNaN(Number(n))) return "—";
    return Math.round(Number(n)).toLocaleString("en-US");
  }

  // === UTXO Rebuild Progress ===
  let utxoRebuildPollInterval = null;

  function initUtxoRebuildPoll() {
    // Override the "Rebuild" button to use background mode
    const rebuildBtn = document.getElementById("btnSnapRefresh");
    if (rebuildBtn) {
      rebuildBtn.textContent = "🔄 Rebuild (background)";
      rebuildBtn.onclick = async () => {
        if (!confirm("Start UTXO rebuild in background? This can take hours. The scan continues normally.")) return;
        try {
          const resp = await api("/api/snapshot/refresh", { method: "POST" });
          toast(resp.message || "Rebuild started in background", "success");
          $("utxoRebuildCard").hidden = false;
          startUtxoRebuildPoll();
        } catch (e) {
          toast("Rebuild error: " + e.message, "error");
        }
      };
    }

    // Check if rebuild is already in progress on load
    pollUtxoRebuildStatus();
    utxoRebuildPollInterval = setInterval(pollUtxoRebuildStatus, 10000);
  }

  async function pollUtxoRebuildStatus() {
    try {
      const data = await api("/api/snapshot/rebuild-status");
      const isRunning = data.running || data.marker_file;
      const progress = data.progress;

      if (isRunning) {
        $("utxoRebuildCard").hidden = false;
        if (progress && progress.message) {
          $("utxoRebuildMsg").textContent = progress.message;
        }
        if (progress && progress.phase === "complete") {
          $("utxoRebuildPill").textContent = "Complete";
          $("utxoRebuildPill").className = "pill";
          $("utxoRebuildPill").style.background = "var(--green)";
          $("utxoRebuildPill").style.color = "#000";
          if ($("utxoRebuildBar")) $("utxoRebuildBar").style.width = "100%";
          setTimeout(() => { $("utxoRebuildCard").hidden = true; }, 30000);
        } else if (progress && progress.phase === "error") {
          $("utxoRebuildPill").textContent = "Error";
          $("utxoRebuildPill").style.background = "var(--red)";
          if ($("utxoRebuildBar")) $("utxoRebuildBar").style.width = "0%";
        } else {
          $("utxoRebuildPill").textContent = "Running";
          $("utxoRebuildPill").className = "pill warn";
          // Simulate progress bar (we don't have exact %, but show activity)
          if ($("utxoRebuildBar")) {
            const existing = parseFloat($("utxoRebuildBar").style.width) || 5;
            $("utxoRebuildBar").style.width = Math.min(existing + 2, 95) + "%";
          }
        }
      } else {
        $("utxoRebuildCard").hidden = true;
        if ($("utxoRebuildBar")) $("utxoRebuildBar").style.width = "0%";
      }
    } catch {}
  }

  function startUtxoRebuildPoll() {
    // Poll more frequently after starting
    pollUtxoRebuildStatus();
  }

  // Init benchmark + UTXO rebuild on boot
  initBenchmarkTab();
  initUtxoRebuildPoll();

  // Initialize global scan state flags
  window.__corpusRunning = false;
  window.__benchRunning = false;
  window.__scanPaused = false;

  // Check for running scans on page load
  (async () => {
    try {
      const [benchData, corpusData, healthData] = await Promise.all([
        api("/api/benchmark/status").catch(() => ({})),
        api("/api/scan/corpus/progress").catch(() => ({})),
        api("/api/health").catch(() => ({})),
      ]);
      if (benchData.running) {
        window.__benchRunning = true;
        $("benchProgress").style.display = "block";
      }
      if (corpusData.running) {
        window.__corpusRunning = true;
      }
      // Check if scan was paused before page reload
      const posFileExists = healthData.scan?.paused || false;
      if (posFileExists) {
        window.__scanPaused = true;
        setScanPill("off", "PAUSED", "Scan paused — click Resume to continue", "");
      }
    } catch {}
  })();
})();
