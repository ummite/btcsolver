/**
 * BTC Solver Dashboard — frontend
 * Real-time WebSocket stats + REST API actions
 */

(() => {
  "use strict";

  // ─── Helpers ──────────────────────────────────────────────────────────────

  const $ = (id) => document.getElementById(id);

  function formatNumber(n) {
    if (n == null || Number.isNaN(n)) return "—";
    if (n >= 1e12) return (n / 1e12).toFixed(2) + " T";
    if (n >= 1e9) return (n / 1e9).toFixed(2) + " G";
    if (n >= 1e6) return (n / 1e6).toFixed(2) + " M";
    if (n >= 1e3) return (n / 1e3).toFixed(1) + " K";
    return String(n);
  }

  function formatDuration(seconds) {
    if (!seconds && seconds !== 0) return "—";
    const s = Math.floor(seconds);
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    if (h > 0) return `${h}h ${String(m).padStart(2, "0")}m ${String(sec).padStart(2, "0")}s`;
    if (m > 0) return `${m}m ${String(sec).padStart(2, "0")}s`;
    return `${sec}s`;
  }

  function toast(msg, type = "") {
    const el = $("toast");
    el.textContent = msg;
    el.className = "toast" + (type ? " " + type : "");
    clearTimeout(toast._t);
    toast._t = setTimeout(() => {
      el.classList.add("hidden");
    }, 4000);
  }

  function setMsg(id, text, type = "") {
    const el = $(id);
    if (!el) return;
    el.textContent = text || "";
    el.className = "form-msg" + (type ? " " + type : "");
  }

  async function api(path, options = {}) {
    const res = await fetch(path, {
      headers: { "Content-Type": "application/json", ...(options.headers || {}) },
      ...options,
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok && data.error) {
      throw new Error(data.error);
    }
    return data;
  }

  // ─── Clock ────────────────────────────────────────────────────────────────

  function tickClock() {
    const now = new Date();
    $("clock").textContent = now.toLocaleTimeString("fr-FR", { hour12: false });
  }
  setInterval(tickClock, 1000);
  tickClock();

  // ─── WebSocket real-time stats ────────────────────────────────────────────

  let ws = null;
  let wsRetry = 0;

  function connectWs() {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    const url = `${proto}//${location.host}/ws`;
    ws = new WebSocket(url);

    ws.onopen = () => {
      wsRetry = 0;
      $("wsStatus").className = "ws-status connected";
      $("wsLabel").textContent = "Temps réel";
    };

    ws.onclose = () => {
      $("wsStatus").className = "ws-status error";
      $("wsLabel").textContent = "Reconnexion…";
      const delay = Math.min(1000 * Math.pow(1.5, wsRetry++), 15000);
      setTimeout(connectWs, delay);
    };

    ws.onerror = () => {
      $("wsStatus").className = "ws-status error";
      $("wsLabel").textContent = "Erreur WS";
    };

    ws.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data);
        if (msg.type === "scan_stats" && msg.data) {
          updateScanStats(msg.data);
        }
      } catch (_) {
        /* ignore */
      }
    };
  }

  function updateScanStats(s) {
    const running = !!s.running;
    const badge = $("scanBadge");
    badge.className = "status-badge " + (running ? "running" : "stopped");
    $("scanBadgeText").textContent = running ? "En cours" : "Arrêté";

    $("btnStart").disabled = running;
    $("btnStop").disabled = !running;

    $("keysPerSec").textContent = formatNumber(s.keys_per_sec) + (s.keys_per_sec ? "/s" : "");
    $("keysTested").textContent = formatNumber(s.keys_tested);
    $("matchesFound").textContent = formatNumber(s.matches_found ?? 0);
    $("elapsed").textContent = formatDuration(s.elapsed_seconds);

    if (s.gpu_util != null) {
      $("gpuUtil").textContent = s.gpu_util.toFixed(0) + "%";
    } else {
      $("gpuUtil").textContent = "—";
    }

    if (s.gpu_temp != null) {
      $("gpuTemp").textContent = s.gpu_temp.toFixed(0) + "°C";
    } else {
      $("gpuTemp").textContent = "—";
    }

    if (s.gpu_vram_used != null && s.gpu_vram_total != null) {
      $("gpuVram").textContent = `${s.gpu_vram_used} / ${s.gpu_vram_total} MB`;
    } else {
      $("gpuVram").textContent = "—";
    }

    if (s.ram_mb != null) {
      $("ramMb").textContent = s.ram_mb.toFixed(0) + " MB";
    } else {
      $("ramMb").textContent = "—";
    }

    $("currentPosition").textContent = s.current_position || "—";
  }

  // ─── Scan controls ────────────────────────────────────────────────────────

  $("scanForm").addEventListener("submit", async (e) => {
    e.preventDefault();
    setMsg("scanMsg", "Démarrage…");

    const body = {
      use_gpu: $("useGpu").checked,
      threads: parseInt($("threads").value, 10) || 23,
      batch_size: parseInt($("batchSize").value, 10) || 256000,
      start_key: $("startKey").value.trim() || null,
      count: parseInt($("count").value, 10) || 0,
      addr_types: $("addrTypes").value.trim() || "legacy,segwit,wrapped,taproot",
      stats_interval: 10,
      progress_interval: 30,
    };

    try {
      const data = await api("/api/scan/start", {
        method: "POST",
        body: JSON.stringify(body),
      });
      setMsg("scanMsg", data.message || "Scan démarré", "success");
      toast(data.message || "Scan démarré", "success");
      $("btnStart").disabled = true;
      $("btnStop").disabled = false;
    } catch (err) {
      setMsg("scanMsg", err.message, "error");
      toast(err.message, "error");
    }
  });

  $("btnStop").addEventListener("click", async () => {
    setMsg("scanMsg", "Arrêt…");
    try {
      const data = await api("/api/scan/stop", { method: "POST" });
      setMsg("scanMsg", data.message || "Scan arrêté", "success");
      toast("Scan arrêté", "success");
    } catch (err) {
      setMsg("scanMsg", err.message, "error");
      toast(err.message, "error");
    }
  });

  // ─── Key checker ──────────────────────────────────────────────────────────

  $("keyForm").addEventListener("submit", async (e) => {
    e.preventDefault();
    const key = $("keyInput").value.trim();
    if (!key) {
      toast("Entrez une clé", "error");
      return;
    }

    $("btnCheck").disabled = true;
    $("btnCheck").textContent = "Vérification…";

    const body = {
      key,
      format: $("keyFormat").value || null,
      passphrase: $("passphrase").value || null,
    };

    try {
      const data = await api("/api/keys/check", {
        method: "POST",
        body: JSON.stringify(body),
      });

      if (data.error) throw new Error(data.error);
      renderKeyResult(data);
      if (data.total_balance_sats > 0) {
        toast(`Solde trouvé : ${data.total_balance_btc} BTC !`, "success");
      } else {
        toast("Aucune balance sur ces adresses", "");
      }
    } catch (err) {
      toast(err.message, "error");
      $("keyResult").classList.add("hidden");
    } finally {
      $("btnCheck").disabled = false;
      $("btnCheck").textContent = "Vérifier";
    }
  });

  function renderKeyResult(data) {
    const el = $("keyResult");
    const r = (data.results && data.results[0]) || {};
    const hasBal = (data.total_balance_sats || 0) > 0;

    el.className = "key-result" + (hasBal ? " has-balance" : "");

    let html = "";
    html += row("Format", r.input_format || "—");
    html += row("Privkey (hex)", r.privkey_hex || "—");
    html += row("Pubkey", r.pubkey_hex || "—");

    if (r.addresses) {
      html += row("Legacy (P2PKH)", r.addresses.legacy);
      html += row("Segwit (P2WPKH)", r.addresses.segwit);
      html += row("Wrapped (P2SH-P2WPKH)", r.addresses.wrapped);
      if (r.addresses.taproot) html += row("Taproot (P2TR)", r.addresses.taproot);
    }

    html += `<div class="result-row"><span class="label">Solde total</span>
      <span class="result-balance">${(data.total_balance_btc || 0).toFixed(8)} BTC
      <span style="font-size:0.75rem;color:var(--text-muted)">(${formatNumber(data.total_balance_sats || 0)} sats)</span></span></div>`;

    if (r.matches && r.matches.length) {
      html += `<div class="match-list">`;
      for (const m of r.matches) {
        html += `<div class="match-item">
          <div class="type">${esc(m.address_type)}</div>
          <div class="mono">${esc(m.address)}</div>
          <div>${m.value_btc.toFixed(8)} BTC (${formatNumber(m.value_sats)} sats)</div>
        </div>`;
      }
      html += `</div>`;
    }

    if (r.error) {
      html += `<div class="form-msg error">${esc(r.error)}</div>`;
    }

    el.innerHTML = html;
  }

  function row(label, value) {
    return `<div class="result-row"><span class="label">${esc(label)}</span><span class="value">${esc(value || "—")}</span></div>`;
  }

  function esc(s) {
    if (s == null) return "";
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  // ─── Bitcoin Core ─────────────────────────────────────────────────────────

  async function refreshBitcoind() {
    try {
      const s = await api("/api/bitcoind/status");
      const running = !!s.running && !s.error;
      const badge = $("btcBadge");
      badge.className = "status-badge " + (running ? "running" : "offline");
      $("btcBadgeText").textContent = running ? "En ligne" : "Hors ligne";

      if (running) {
        $("btcBlocks").textContent = formatNumber(s.blocks);
        $("btcHeaders").textContent = formatNumber(s.headers);
        $("btcSync").textContent = (s.sync_percentage ?? 0).toFixed(2) + "%";
        $("btcPeers").textContent = s.connections ?? "—";
        const pct = Math.min(100, s.verification_progress ?? s.sync_percentage ?? 0);
        $("btcProgress").style.width = pct + "%";
        $("btcProgressLabel").textContent = pct.toFixed(1) + "%";
      } else {
        $("btcBlocks").textContent = "—";
        $("btcHeaders").textContent = "—";
        $("btcSync").textContent = "—";
        $("btcPeers").textContent = "—";
        $("btcProgress").style.width = "0%";
        $("btcProgressLabel").textContent = "0%";
      }
    } catch (_) {
      $("btcBadge").className = "status-badge offline";
      $("btcBadgeText").textContent = "Hors ligne";
    }
  }

  $("btnBtcStart").addEventListener("click", async () => {
    setMsg("btcMsg", "Démarrage de bitcoind (peut prendre ~30s)…");
    $("btnBtcStart").disabled = true;
    try {
      const data = await api("/api/bitcoind/start", { method: "POST" });
      if (data.error) throw new Error(data.error);
      setMsg("btcMsg", data.message || "Démarré", "success");
      toast("Bitcoin Core démarré", "success");
      await refreshBitcoind();
    } catch (err) {
      setMsg("btcMsg", err.message, "error");
      toast(err.message, "error");
    } finally {
      $("btnBtcStart").disabled = false;
    }
  });

  $("btnBtcStop").addEventListener("click", async () => {
    setMsg("btcMsg", "Arrêt…");
    try {
      const data = await api("/api/bitcoind/stop", { method: "POST" });
      if (data.error) throw new Error(data.error);
      setMsg("btcMsg", data.message || "Arrêté", "success");
      toast("Bitcoin Core arrêté", "success");
      await refreshBitcoind();
    } catch (err) {
      setMsg("btcMsg", err.message, "error");
      toast(err.message, "error");
    }
  });

  // ─── Snapshot ─────────────────────────────────────────────────────────────

  async function refreshSnapshot() {
    try {
      const s = await api("/api/snapshot/info");
      if (s.error) throw new Error(s.error);

      const fresh = !!s.fresh;
      $("snapBadge").className = "status-badge " + (fresh ? "fresh" : "stale");
      $("snapBadgeText").textContent = fresh ? "À jour" : "Obsolète";
      $("snapAge").textContent =
        s.age_hours != null ? s.age_hours.toFixed(1) + " h" : "—";
      $("snapSize").textContent =
        s.size_mb != null ? s.size_mb.toFixed(1) + " MB" : "—";
      $("snapMaxAge").textContent =
        s.max_age_hours != null ? s.max_age_hours.toFixed(0) + " h" : "—";
      $("snapPath").textContent = s.path || "—";
    } catch (err) {
      $("snapBadge").className = "status-badge stale";
      $("snapBadgeText").textContent = "Erreur";
      $("snapAge").textContent = "—";
    }
  }

  $("btnSnapRefresh").addEventListener("click", async () => {
    if (!confirm("Régénérer le snapshot UTXO ? Cela peut prendre plusieurs minutes.")) return;
    setMsg("snapMsg", "Indexation en cours… (ne fermez pas l'onglet)");
    $("btnSnapRefresh").disabled = true;
    try {
      const data = await api("/api/snapshot/refresh", { method: "POST" });
      if (data.error) throw new Error(data.error);
      setMsg("snapMsg", "Snapshot régénéré. Redémarrez le dashboard pour recharger l'index.", "success");
      toast("Snapshot régénéré", "success");
      await refreshSnapshot();
    } catch (err) {
      setMsg("snapMsg", err.message, "error");
      toast(err.message, "error");
    } finally {
      $("btnSnapRefresh").disabled = false;
    }
  });

  // ─── Health / periodic refresh ────────────────────────────────────────────

  async function refreshHealth() {
    try {
      const h = await api("/api/system/health");
      $("indexLoaded").textContent = h.index_loaded ? "Oui" : "Non";
      $("footerHealth").textContent = h.status === "ok" ? "API OK" : "API ?";
      if (h.scan && !h.scan.error) updateScanStats(h.scan);
    } catch (_) {
      $("footerHealth").textContent = "API offline";
    }
  }

  // ─── Init ─────────────────────────────────────────────────────────────────

  connectWs();
  refreshBitcoind();
  refreshSnapshot();
  refreshHealth();

  setInterval(refreshBitcoind, 15000);
  setInterval(refreshSnapshot, 30000);
  setInterval(refreshHealth, 20000);
})();
