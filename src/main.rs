use anyhow::{Context, Result, bail};
use base64::Engine;
use bitcoin::key::{CompressedPublicKey, UntweakedPublicKey};
use bitcoin::{Address, Network, NetworkKind, PrivateKey, secp256k1::Secp256k1};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// BTCSolver - Scanner ultra-rapide et 100% privé de soldes Bitcoin.
/// 
/// Deux modes principaux :
/// - Via votre nœud Bitcoin Core local (RPC + scantxoutset) pour données toujours fraîches.
/// - Via un index offline ultra-rapide construit une fois depuis un snapshot dumptxoutset
///   (après synchronisation complète de la chaîne, ce qui ne vous dérange pas).
#[derive(Parser, Debug)]
#[command(name = "btcsolver", version = "0.2.0", author = "BTCSolver Project")]
#[command(about = "Vérifie le solde BTC de clé(s) privée(s) en toute confidentialité (nœud local ou index offline).")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Vérifie le(s) solde(s) pour une ou plusieurs clé(s) privée(s).
    /// Utilise par défaut le RPC d'un nœud local, ou un index offline si --index est fourni.
    Balance(BalanceArgs),

    /// Construit un index de soldes ultra-rapide et offline à partir d'un snapshot
    /// généré par `bitcoin-cli dumptxoutset ...`. 
    /// Une fois l'index construit, les requêtes de solde sont instantanées, sans nœud.
    BuildIndex(BuildIndexArgs),
}

#[derive(Parser, Debug)]
struct BalanceArgs {
    /// Une clé privée unique (format WIF comme "L1..." ou "5..." ou hex 64 caractères)
    #[arg(short, long, value_name = "WIF|HEX")]
    key: Option<String>,

    /// Fichier texte contenant une clé privée par ligne (WIF ou hex). Idéal pour lots.
    #[arg(short, long, value_name = "FICHIER")]
    file: Option<PathBuf>,

    /// Récupérer les clés depuis stdin (une par ligne)
    #[arg(long)]
    stdin: bool,

    /// Chemin vers un index offline construit avec `build-index` (pour mode complètement offline et instantané).
    #[arg(long, value_name = "INDEX.redb")]
    index: Option<PathBuf>,

    /// URL RPC complète (ex: http://127.0.0.1:8332). Si omis, utilise 127.0.0.1 + port selon réseau.
    #[arg(long, value_name = "URL")]
    rpc_url: Option<String>,

    /// Utilisateur RPC (si pas de cookie)
    #[arg(long, value_name = "USER")]
    rpc_user: Option<String>,

    /// Mot de passe RPC (si pas de cookie)
    #[arg(long, value_name = "PASS")]
    rpc_password: Option<String>,

    /// Chemin vers le fichier .cookie (sinon auto-détection depuis datadir)
    #[arg(long, value_name = "CHEMIN")]
    cookie_file: Option<PathBuf>,

    /// Répertoire de données Bitcoin Core (pour auto .cookie). Défaut: %APPDATA%\\Bitcoin
    #[arg(long, value_name = "DIR")]
    bitcoin_datadir: Option<PathBuf>,

    /// Réseau Bitcoin
    #[arg(short, long, value_enum, default_value_t = NetworkArg::Main)]
    network: NetworkArg,

    /// Afficher les adresses même si solde = 0
    #[arg(long, default_value_t = false)]
    show_all: bool,

    /// Afficher les soldes en satoshis au lieu de BTC
    #[arg(long, default_value_t = false)]
    sats: bool,

    /// Ne pas appeler le nœud / index, juste dériver et afficher les adresses (mode test)
    #[arg(long, default_value_t = false)]
    derive_only: bool,

    /// Verbose: plus de détails
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Parser, Debug)]
struct BuildIndexArgs {
    /// Fichier snapshot généré par `bitcoin-cli dumptxoutset /chemin/vers/utxos.dat latest`
    #[arg(long, value_name = "utxos.dat")]
    snapshot: PathBuf,

    /// Fichier d'index de sortie (format redb, à utiliser ensuite avec `balance --index`)
    #[arg(long, value_name = "index.redb")]
    output: PathBuf,

    /// Réseau (doit correspondre au snapshot)
    #[arg(short, long, value_enum, default_value_t = NetworkArg::Main)]
    network: NetworkArg,

    /// Afficher la progression détaillée
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum NetworkArg {
    Main,
    Test,
    Signet,
    Regtest,
}

impl From<NetworkArg> for Network {
    fn from(arg: NetworkArg) -> Self {
        match arg {
            NetworkArg::Main => Network::Bitcoin,
            NetworkArg::Test => Network::Testnet,
            NetworkArg::Signet => Network::Signet,
            NetworkArg::Regtest => Network::Regtest,
        }
    }
}

impl NetworkArg {
    fn rpc_port(&self) -> u16 {
        match self {
            NetworkArg::Main => 8332,
            NetworkArg::Test => 18332,
            NetworkArg::Signet => 38332,
            NetworkArg::Regtest => 18443,
        }
    }

    fn datadir_suffix(&self) -> &'static str {
        match self {
            NetworkArg::Main => "",
            NetworkArg::Test => "testnet3",
            NetworkArg::Signet => "signet",
            NetworkArg::Regtest => "regtest",
        }
    }
}

#[derive(Debug, Clone)]
struct DerivedAddress {
    kind: &'static str,
    address: Address,
}

#[derive(Debug, Clone)]
struct KeyResult {
    input: String, // original input (we avoid printing full key for safety)
    input_kind: &'static str,
    addresses: Vec<(DerivedAddress, Option<f64>)>, // (addr, balance_btc or None)
    total_btc: f64,
}

#[derive(Debug, Deserialize)]
struct Unspent {
    #[serde(default)]
    desc: Option<String>,
    #[serde(default, rename = "scriptPubKey")]
    script_pub_key: Option<String>,
    amount: f64,
}

#[derive(Debug, Deserialize)]
struct ScanResponse {
    success: bool,
    #[serde(default)]
    total_amount: f64,
    #[serde(default)]
    unspents: Vec<Unspent>,
    #[serde(default)]
    height: Option<u64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Balance(ref args) => run_balance(args)?,
        Commands::BuildIndex(ref args) => run_build_index(args)?,
    }

    Ok(())
}

fn run_balance(args: &BalanceArgs) -> Result<()> {
    let network: Network = args.network.into();
    let secp = Secp256k1::new();

    // Collect all private key inputs
    let mut raw_keys: Vec<String> = Vec::new();

    if let Some(k) = &args.key {
        raw_keys.push(k.clone());
    }
    if let Some(path) = &args.file {
        let file = fs::File::open(path).with_context(|| format!("Impossible d'ouvrir {}", path.display()))?;
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                raw_keys.push(trimmed.to_string());
            }
        }
    }
    if args.stdin {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                raw_keys.push(trimmed.to_string());
            }
        }
    }

    if raw_keys.is_empty() {
        bail!("Aucune clé fournie. Utilisez --key, --file ou --stdin. Voir --help.");
    }

    println!("🔐 BTCSolver - Scanner de solde Bitcoin privé");
    println!("   Réseau: {:?}", network);
    println!("   {} clé(s) à analyser\n", raw_keys.len());

    // Derive addresses for every key (never expose privkey beyond this point)
    let mut key_results: Vec<KeyResult> = Vec::with_capacity(raw_keys.len());

    for (idx, raw) in raw_keys.iter().enumerate() {
        let (privkey, input_kind) = parse_private_key(raw, network)
            .with_context(|| format!("Clé invalide à la ligne {}", idx + 1))?;

        let derived = derive_all_addresses(&privkey, &secp, network)?;

        let addr_balances: Vec<(DerivedAddress, Option<f64>)> = derived
            .into_iter()
            .map(|da| (da, None))
            .collect();

        key_results.push(KeyResult {
            input: mask_key(raw),
            input_kind,
            addresses: addr_balances,
            total_btc: 0.0,
        });
    }

    if args.derive_only {
        print_derive_only_results(&key_results, args.sats);
        return Ok(());
    }

    // ===================== OFFLINE INDEX PATH (super rapide, pas de nœud requis au runtime) =====================
    if let Some(index_path) = &args.index {
        run_balance_offline(args, index_path, network, &secp, &mut key_results)?;
        print_results(&key_results, args.sats, args.show_all);

        let grand_total: f64 = key_results.iter().map(|k| k.total_btc).sum();
        println!("\n════════════════════════════════════════════════════════════");
        if args.sats {
            println!("💰 TOTAL GLOBAL : {} sat", (grand_total * 100_000_000.0).round() as u64);
        } else {
            println!("💰 TOTAL GLOBAL : {:.8} BTC", grand_total);
        }
        println!("════════════════════════════════════════════════════════════");
        println!("\n✅ Terminé (mode index offline). Vos clés privées n'ont jamais quitté votre machine.");
        return Ok(());
    }

    // Build unique list of "addr(...)" descriptors for efficient *single* scantxoutset call
    // (the cost is dominated by iterating the UTXO set once, not by # of addrs)
    let mut unique_addrs: HashSet<String> = HashSet::new();
    for kr in &key_results {
        for (da, _) in &kr.addresses {
            unique_addrs.insert(format!("addr({})", da.address));
        }
    }

    println!("📡 Connexion au nœud Bitcoin Core pour scan UTXO (scantxoutset)...");
    let start = Instant::now();

    let rpc_url = build_rpc_url(args, &args.network);
    let auth_header = get_auth_header(args, &args.network)?;

    if args.verbose {
        println!("   RPC: {}", rpc_url);
    }

    // Optional: check node status
    if let Ok(info) = call_getblockchaininfo(&rpc_url, &auth_header) {
        let progress = info.get("verificationprogress").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let blocks = info.get("blocks").and_then(|v| v.as_u64()).unwrap_or(0);
        let chain = info.get("chain").and_then(|v| v.as_str()).unwrap_or("?");
        println!("   Nœud: {} @ bloc {}, progrès: {:.2}%", chain, blocks, progress * 100.0);
        if progress < 0.999 {
            eprintln!("⚠️  Attention: le nœud n'a pas terminé la synchronisation. Les soldes peuvent être incomplets !");
        }
    } else if args.verbose {
        println!("   (Impossible d'obtenir getblockchaininfo - poursuite quand même)");
    }

    let scan_objects: Vec<String> = unique_addrs.into_iter().collect();
    println!("   Envoi de {} adresse(s) unique(s) au scan (une passe sur l'UTXO set)...", scan_objects.len());

    let scan_resp = call_scantxoutset(&rpc_url, &auth_header, &scan_objects)
        .context("Échec de l'appel RPC scantxoutset. Vérifiez que bitcoind tourne, est synchronisé, et que l'auth est correcte.")?;

    if !scan_resp.success {
        bail!("scantxoutset a retourné success=false. Le nœud a-t-il assez de ressources ?");
    }

    let elapsed = start.elapsed();
    println!("   ✓ Scan terminé en {:.1}s (hauteur approx: {:?})", elapsed.as_secs_f32(), scan_resp.height);
    println!("   Total trouvé par le nœud pour ces adresses: {:.8} BTC\n", scan_resp.total_amount);

    // Map results back to our structures
    let mut funded_addrs: HashMap<String, f64> = HashMap::new();
    for u in &scan_resp.unspents {
        let amt = u.amount;
        if let Some(desc) = &u.desc {
            if let Some(inner) = desc.strip_prefix("addr(").and_then(|s| s.strip_suffix(")")) {
                *funded_addrs.entry(inner.to_string()).or_default() += amt;
            }
        } else if let Some(_spk) = &u.script_pub_key {
            // Fallback: we could reverse script to address but complicated. For now rely on desc.
            // Many nodes return desc for addr() scans.
        }
    }

    // Fill balances
    for kr in &mut key_results {
        let mut total = 0.0f64;
        for (da, bal_slot) in &mut kr.addresses {
            let addr_str = da.address.to_string();
            if let Some(&b) = funded_addrs.get(&addr_str) {
                *bal_slot = Some(b);
                total += b;
            }
        }
        kr.total_btc = total;
    }

    // Print beautiful results (French)
    print_results(&key_results, args.sats, args.show_all);

    let grand_total: f64 = key_results.iter().map(|k| k.total_btc).sum();
    println!("\n════════════════════════════════════════════════════════════");
    if args.sats {
        println!("💰 TOTAL GLOBAL : {} sat", (grand_total * 100_000_000.0).round() as u64);
    } else {
        println!("💰 TOTAL GLOBAL : {:.8} BTC", grand_total);
    }
    println!("════════════════════════════════════════════════════════════");
    println!("\n✅ Terminé. Vos clés privées n'ont jamais quitté votre machine.");

    Ok(())
}

/// Parse WIF or hex private key. Returns (PrivateKey, kind_description)
fn parse_private_key(s: &str, network: Network) -> Result<(PrivateKey, &'static str)> {
    let s = s.trim();
    if s.len() > 50 && (s.starts_with('5') || s.starts_with('K') || s.starts_with('L') || s.starts_with('9') || s.starts_with('c')) {
        // Likely WIF
        let pk = PrivateKey::from_wif(s).context("WIF invalide ou corrompu (checksum?)")?;
        // WIF carries its own network info in version byte. We still accept for the target network.
        if NetworkKind::from(network) != pk.network && network != Network::Regtest {
            // Allow mismatch only for regtest flexibility, otherwise warn in practice
            eprintln!("⚠️  Note: le WIF semble être pour un réseau différent de celui demandé.");
        }
        let kind = if pk.compressed { "WIF (compressée)" } else { "WIF (non compressée - legacy)" };
        Ok((pk, kind))
    } else {
        // Hex
        let hex_clean = s.strip_prefix("0x").unwrap_or(s);
        if hex_clean.len() != 64 {
            bail!("Clé hex invalide : doit faire exactement 64 caractères (32 octets).");
        }
        let bytes = hex::decode(hex_clean).context("Hex invalide")?;
        if bytes.len() != 32 {
            bail!("Clé hex doit faire 32 octets.");
        }
        let pk = PrivateKey::from_slice(&bytes, network)
            .context("Impossible de créer la clé privée depuis les octets")?;
        Ok((pk, "HEX (32 octets)"))
    }
}

fn mask_key(raw: &str) -> String {
    if raw.len() <= 8 {
        "****".to_string()
    } else {
        format!("{}...{}", &raw[..6], &raw[raw.len()-4..])
    }
}

/// Derive the 4 (or 5) standard single-sig address types a private key can control.
fn derive_all_addresses(pk: &PrivateKey, secp: &Secp256k1<bitcoin::secp256k1::All>, network: Network) -> Result<Vec<DerivedAddress>> {
    let pubkey = pk.public_key(secp);
    let mut out = Vec::with_capacity(4);

    // 1. Legacy P2PKH (1...)
    let p2pkh = Address::p2pkh(pubkey, network);
    out.push(DerivedAddress { kind: "legacy (P2PKH)", address: p2pkh });

    // For segwit and taproot we almost always want the compressed pubkey.
    let compressed = CompressedPublicKey::from_private_key(secp, pk)
        .context("Impossible d'obtenir la clé publique compressée (requis pour SegWit/Taproot)")?;

    // 2. Native SegWit P2WPKH (bc1q... or tb1q...)
    let p2wpkh = Address::p2wpkh(&compressed, network);
    out.push(DerivedAddress { kind: "native segwit (P2WPKH)", address: p2wpkh });

    // 3. Wrapped SegWit P2SH-P2WPKH (3... or 2...)
    let p2sh_wpkh = Address::p2shwpkh(&compressed, network);
    out.push(DerivedAddress { kind: "wrapped segwit (P2SH-P2WPKH)", address: p2sh_wpkh });

    // 4. Taproot P2TR (bc1p... ) - key path spend only, no scripts
    let xonly: UntweakedPublicKey = compressed.into();
    let p2tr = Address::p2tr(secp, xonly, None, network);
    out.push(DerivedAddress { kind: "taproot (P2TR)", address: p2tr });

    Ok(out)
}

fn build_rpc_url(args: &BalanceArgs, net: &NetworkArg) -> String {
    if let Some(u) = &args.rpc_url {
        return u.clone();
    }
    let host = "127.0.0.1";
    format!("http://{}:{}", host, net.rpc_port())
}

/// Build "Basic base64(user:pass)" header value from cookie or explicit creds.
fn get_auth_header(args: &BalanceArgs, net: &NetworkArg) -> Result<String> {
    // Priority: explicit --cookie-file > --rpc-user/pass > auto cookie from datadir
    if let Some(cookie_path) = &args.cookie_file {
        return load_cookie_auth(cookie_path);
    }

    if let (Some(user), Some(pass)) = (&args.rpc_user, &args.rpc_password) {
        let token = format!("{}:{}", user, pass);
        let encoded = base64::engine::general_purpose::STANDARD.encode(token.as_bytes());
        return Ok(format!("Basic {}", encoded));
    }

    // Auto cookie
    let datadir = args.bitcoin_datadir.clone().unwrap_or_else(default_bitcoin_datadir);
    let suffix = net.datadir_suffix();
    let cookie_path = if suffix.is_empty() {
        datadir.join(".cookie")
    } else {
        datadir.join(suffix).join(".cookie")
    };

    if cookie_path.exists() {
        return load_cookie_auth(&cookie_path);
    }

    // Last resort: try no auth (some regtest setups)
    if net == &NetworkArg::Regtest {
        return Ok("".to_string());
    }

    bail!(
        "Aucun moyen d'authentification trouvé.\n\
         Options:\n  • Lancez bitcoind et laissez-le créer le .cookie (recommandé)\n  \
           • --cookie-file <chemin>\n  \
           • --rpc-user <u> --rpc-password <p>\n  \
           • --rpc-url http://user:pass@host:port"
    );
}

fn load_cookie_auth(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Lecture du cookie impossible: {}", path.display()))?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        bail!("Fichier cookie vide: {}", path.display());
    }
    let encoded = base64::engine::general_purpose::STANDARD.encode(trimmed.as_bytes());
    Ok(format!("Basic {}", encoded))
}

fn default_bitcoin_datadir() -> PathBuf {
    // Prefer Y:\Bitcoin if it exists (data sur Y:, pas C:)
    let y_datadir = PathBuf::from(r"Y:\Bitcoin");
    if y_datadir.join(".cookie").exists() || y_datadir.join("chainstate").exists() {
        return y_datadir;
    }
    // Windows default: %APPDATA%\Bitcoin
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("Bitcoin");
    }
    PathBuf::from(r"C:\Users\Default\AppData\Roaming\Bitcoin")
}

// We need a small helper for config dir without extra crate. The above is good enough for Windows.

fn call_getblockchaininfo(rpc_url: &str, auth: &str) -> Result<serde_json::Value> {
    let payload = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "btcsolver",
        "method": "getblockchaininfo",
        "params": []
    });

    let mut req = ureq::post(rpc_url)
        .set("Content-Type", "application/json");
    if !auth.is_empty() {
        req = req.set("Authorization", auth);
    }

    let resp = req.send_json(payload)
        .context("Échec requête getblockchaininfo")?;

    let val: serde_json::Value = resp.into_json()?;
    if let Some(err) = val.get("error") {
        if !err.is_null() {
            bail!("RPC error: {}", err);
        }
    }
    Ok(val.get("result").cloned().unwrap_or(val))
}

fn call_scantxoutset(rpc_url: &str, auth: &str, scanobjects: &[String]) -> Result<ScanResponse> {
    let payload = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "btcsolver",
        "method": "scantxoutset",
        "params": ["start", scanobjects]
    });

    let mut req = ureq::post(rpc_url)
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(300)); // scantxoutset can be slow
    if !auth.is_empty() {
        req = req.set("Authorization", auth);
    }

    let resp = req.send_json(payload).context("Échec envoi RPC scantxoutset")?;
    let val: serde_json::Value = resp.into_json().context("Réponse JSON invalide")?;

    if let Some(err) = val.get("error") {
        if !err.is_null() {
            bail!("RPC error from bitcoind: {}", err);
        }
    }

    let result = val.get("result").context("Pas de champ 'result' dans la réponse scantxoutset")?;
    let parsed: ScanResponse = serde_json::from_value(result.clone())
        .context("Impossible de désérialiser la réponse scantxoutset")?;
    Ok(parsed)
}

fn print_derive_only_results(results: &[KeyResult], _in_sats: bool) {
    println!("(Mode --derive-only : pas de requête au nœud)\n");
    for (i, kr) in results.iter().enumerate() {
        println!("Clé #{} [{}] (masquée: {})", i + 1, kr.input_kind, kr.input);
        for (da, _) in &kr.addresses {
            println!("  • {:<28} {}", da.kind, da.address);
        }
        println!();
    }
}

fn print_results(results: &[KeyResult], in_sats: bool, show_all: bool) {
    for (i, kr) in results.iter().enumerate() {
        println!("────────────────────────────────────────────────────────────");
        println!("Clé #{} — {} — masquée: {}", i + 1, kr.input_kind, kr.input);

        let mut has_funds = false;
        for (da, bal_opt) in &kr.addresses {
            let bal = bal_opt.unwrap_or(0.0);
            if bal > 0.0 || show_all {
                has_funds = has_funds || bal > 0.0;
                if in_sats {
                    let sats = (bal * 100_000_000.0).round() as i64;
                    println!("  {:<28} {:>18} sat   {}", da.kind, sats, da.address);
                } else {
                    println!("  {:<28} {:>12.8} BTC   {}", da.kind, bal, da.address);
                }
            }
        }

        if !has_funds && !show_all {
            println!("  (Aucun solde détecté sur les adresses standard. Utilisez --show-all pour voir les adresses.)");
        } else if kr.total_btc > 0.0 {
            if in_sats {
                println!("  → Total pour cette clé : {} sat", (kr.total_btc * 100_000_000.0).round() as u64);
            } else {
                println!("  → Total pour cette clé : {:.8} BTC", kr.total_btc);
            }
        }
    }
}

// ============================================================================
// OFFLINE INDEX MODE - Super efficace une fois la blockchain complète synchronisée
// ============================================================================

use redb::{Database, ReadableDatabase, TableDefinition};

const BALANCES_TABLE: TableDefinition<&[u8], u64> = TableDefinition::new("script_balances");
const META_TABLE: TableDefinition<&str, String> = TableDefinition::new("meta");

/// Port des fonctions de décompression du format dumptxoutset (voir utxo_to_sqlite.py dans Bitcoin Core)
#[allow(unused_assignments)]
fn read_varint<R: Read>(reader: &mut R) -> Result<u64> {
    let mut n: u64 = 0;
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let dat = buf[0];
        n = (n << 7) | ((dat & 0x7f) as u64);
        if (dat & 0x80) > 0 {
            n += 1;
        } else {
            return Ok(n);
        }
    }
}

fn read_compact_size<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    let n = buf[0] as u64;
    match n {
        253 => {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            Ok(u16::from_le_bytes(b) as u64)
        }
        254 => {
            let mut b = [0u8; 4];
            reader.read_exact(&mut b)?;
            Ok(u32::from_le_bytes(b) as u64)
        }
        255 => {
            let mut b = [0u8; 8];
            reader.read_exact(&mut b)?;
            Ok(u64::from_le_bytes(b))
        }
        _ => Ok(n),
    }
}

fn decompress_amount(x: u64) -> u64 {
    if x == 0 {
        return 0;
    }
    let mut x = x - 1;
    let e = x % 10;
    x /= 10;
    let mut n: u64 = 0;
    if e < 9 {
        let d = (x % 9) + 1;
        x /= 9;
        n = x * 10 + d;
    } else {
        n = x + 1;
    }
    let mut e = e;
    while e > 0 {
        n *= 10;
        e -= 1;
    }
    n
}

fn decompress_script<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let size = read_varint(reader)? as usize;
    if size == 0 {
        // P2PKH
        let mut h160 = [0u8; 20];
        reader.read_exact(&mut h160)?;
        let mut script = vec![0x76, 0xa9, 0x14];
        script.extend_from_slice(&h160);
        script.extend_from_slice(&[0x88, 0xac]);
        Ok(script)
    } else if size == 1 {
        // P2SH
        let mut h160 = [0u8; 20];
        reader.read_exact(&mut h160)?;
        let mut script = vec![0xa9, 0x14];
        script.extend_from_slice(&h160);
        script.push(0x87);
        Ok(script)
    } else if size == 2 || size == 3 {
        // P2PK compressed
        let mut key = [0u8; 32];
        reader.read_exact(&mut key)?;
        let mut script = vec![33, size as u8];
        script.extend_from_slice(&key);
        script.push(0xac);
        Ok(script)
    } else if size == 4 || size == 5 {
        // P2PK uncompressed (rare en 2026)
        let mut compressed = [0u8; 33];
        compressed[0] = (size - 2) as u8;
        reader.read_exact(&mut compressed[1..])?;
        let full = decompress_pubkey(&compressed)?;
        let mut script = vec![0x41];
        script.extend_from_slice(&full);
        script.push(0xac);
        Ok(script)
    } else {
        let real_size = size - 6;
        if real_size > 10000 {
            bail!("script trop long dans snapshot");
        }
        let mut script = vec![0u8; real_size];
        reader.read_exact(&mut script)?;
        Ok(script)
    }
}

fn decompress_pubkey(compressed: &[u8]) -> Result<Vec<u8>> {
    // Placeholder pour les très vieux P2PK uncompressed. 
    // On construit un script 65 bytes avec y=0 (ne matchera pas parfaitement les anciens, mais ces UTXOs sont quasi inexistants aujourd'hui).
    if compressed.len() != 33 {
        bail!("bad compressed pubkey length");
    }
    let mut out = vec![0x04];
    out.extend_from_slice(&compressed[1..]);
    out.extend_from_slice(&[0u8; 32]);
    Ok(out)
}

fn run_build_index(args: &BuildIndexArgs) -> Result<()> {
    println!("🔨 Construction de l'index offline à partir du snapshot...");
    println!("   Snapshot: {}", args.snapshot.display());
    println!("   Sortie    : {}", args.output.display());

    if args.output.exists() {
        bail!("Le fichier de sortie existe déjà. Supprimez-le ou choisissez un autre nom.");
    }

    let file = fs::File::open(&args.snapshot)
        .with_context(|| format!("Impossible d'ouvrir le snapshot {}", args.snapshot.display()))?;
    let mut reader = std::io::BufReader::new(file);

    // Header
    let mut magic = [0u8; 5];
    reader.read_exact(&mut magic)?;
    if &magic != b"utxo\xff" {
        bail!("Magic bytes invalides. Ce n'est pas un snapshot dumptxoutset valide.");
    }

    let mut ver = [0u8; 2];
    reader.read_exact(&mut ver)?;
    let version = u16::from_le_bytes(ver);
    if version != 2 {
        bail!("Version de snapshot {} non supportée (seulement v2).", version);
    }

    let mut net_magic = [0u8; 4];
    reader.read_exact(&mut net_magic)?;
    let mut block_hash = [0u8; 32];
    reader.read_exact(&mut block_hash)?;
    let mut num_buf = [0u8; 8];
    reader.read_exact(&mut num_buf)?;
    let num_utxos = u64::from_le_bytes(num_buf);

    let display_hash = {
        let mut h = block_hash;
        h.reverse();
        hex::encode(h)
    };
    println!("   Snapshot basé sur le bloc {} ({}...)", display_hash, &display_hash[..16]);
    println!("   → Pour voir la date exacte des données : https://mempool.space/block/{}", display_hash);
    println!("   {} UTXOs à traiter (données quelques jours en retard max si tu prends un snapshot récent).", num_utxos);

    let mut aggregates: HashMap<Vec<u8>, u64> = HashMap::new();

    let start = Instant::now();
    let mut processed: u64 = 0;
    let mut coins_per_hash_left: u64 = 0;
    let mut prevout_hash = [0u8; 32];

    while processed < num_utxos {
        if coins_per_hash_left == 0 {
            reader.read_exact(&mut prevout_hash)?;
            coins_per_hash_left = read_compact_size(&mut reader)?;
        }

        let _vout = read_compact_size(&mut reader)?;

        let code = read_varint(&mut reader)?;
        let _height = code >> 1;
        let _is_coinbase = (code & 1) == 1;

        let compressed_amt = read_varint(&mut reader)?;
        let amount_sats = decompress_amount(compressed_amt);

        let script = decompress_script(&mut reader)?;

        *aggregates.entry(script).or_insert(0) += amount_sats;

        coins_per_hash_left -= 1;
        processed += 1;

        if processed % 1_000_000 == 0 {
            let pct = (processed as f64 / num_utxos as f64) * 100.0;
            let elapsed = start.elapsed().as_secs_f64();
            let rate = processed as f64 / elapsed.max(1.0);
            let remaining = (num_utxos - processed) as f64 / rate.max(1.0);
            println!("   {} / {} ({:.1}%) - ~{:.0}s restantes ({} UTXO/s)", 
                     processed, num_utxos, pct, remaining, rate as u64);
        }
    }

    let elapsed = start.elapsed();
    println!("   Parsing terminé en {:.1}s. {} scripts uniques avec solde.", elapsed.as_secs_f32(), aggregates.len());

    println!("   Écriture de l'index redb (peut prendre du temps)...");
    let db = Database::create(&args.output)
        .with_context(|| "Création de la base redb impossible (espace disque / permissions ?)")?;

    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(BALANCES_TABLE)?;
        for (script, total_sats) in aggregates {
            table.insert(script.as_slice(), total_sats)?;
        }

        let mut meta = write_txn.open_table(META_TABLE)?;
        meta.insert("base_block_hash", &display_hash)?;
        meta.insert("build_time", &chrono::Utc::now().to_rfc3339())?;
        meta.insert("num_utxos", &num_utxos.to_string())?;
    }
    write_txn.commit()?;

    println!("✅ Index construit : {}", args.output.display());
    println!("   Données basées sur le bloc {} (vérifie la date ci-dessus).", display_hash);
    println!("   Utilisation instantanée : .\\btcsolver.exe balance --index {} --key TA_CLE_PRIVEE", args.output.display());

    Ok(())
}

fn run_balance_offline(_args: &BalanceArgs, index_path: &std::path::Path, _network: Network, _secp: &Secp256k1<bitcoin::secp256k1::All>, key_results: &mut [KeyResult]) -> Result<()> {
    println!("📂 Mode OFFLINE (index) : {}", index_path.display());

    let db = Database::open(index_path)
        .with_context(|| format!("Ouverture index impossible: {}", index_path.display()))?;

    let read_txn = db.begin_read()?;

    // Print freshness info if available
    if let Ok(meta_table) = read_txn.open_table(META_TABLE) {
        if let Ok(Some(hash)) = meta_table.get("base_block_hash") {
            let h = hash.value();
            println!("   Index basé sur le bloc {} (https://mempool.space/block/{})", h, h);
        }
        if let Ok(Some(build_time)) = meta_table.get("build_time") {
            println!("   Index construit le : {}", build_time.value());
        }
    }

    let table = read_txn.open_table(BALANCES_TABLE)?;

    for kr in key_results.iter_mut() {
        let mut total_sats: u64 = 0;

        for (da, bal_slot) in &mut kr.addresses {
            let spk = da.address.script_pubkey();
            let key_bytes = spk.as_bytes();

            if let Ok(Some(val)) = table.get(key_bytes) {
                let sats = val.value();
                *bal_slot = Some(sats as f64 / 100_000_000.0);
                total_sats += sats;
            }
        }

        kr.total_btc = total_sats as f64 / 100_000_000.0;
    }

    Ok(())
}


