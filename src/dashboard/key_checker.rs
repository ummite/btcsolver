use anyhow::Result;
use bip39::Mnemonic;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::{Address, Network};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;

use crate::flat_index::FlatIndex;

/// Result of checking a single private key
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyCheckResult {
    pub input: String,
    pub input_format: String,
    pub privkey_hex: String,
    pub pubkey_hex: String,
    pub addresses: KeyAddresses,
    pub matches: Vec<UTXOMatch>,
    pub total_balance_sats: u64,
    pub total_balance_btc: f64,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyAddresses {
    pub legacy: String,   // P2PKH
    pub segwit: String,   // P2WPKH
    pub wrapped: String,  // P2SH-P2WPKH
    pub taproot: String,  // P2TR
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UTXOMatch {
    pub address: String,
    pub address_type: String,
    pub value_sats: u64,
    pub value_btc: f64,
    pub script_hex: String,
}

/// Detect input format
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum KeyFormat {
    Hex,
    WIF,
    BIP39,
    Brainwallet,
}

impl std::fmt::Display for KeyFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyFormat::Hex => write!(f, "hex"),
            KeyFormat::WIF => write!(f, "WIF"),
            KeyFormat::BIP39 => write!(f, "BIP39"),
            KeyFormat::Brainwallet => write!(f, "brainwallet"),
        }
    }
}

pub struct KeyChecker;

impl KeyChecker {
    /// Detect the format of the input string
    pub fn detect_format(input: &str) -> Result<KeyFormat> {
        let trimmed = input.trim();

        // WIF: starts with '5', 'K', or 'L' (mainnet), base58, 51-52 chars
        if (trimmed.starts_with('5') || trimmed.starts_with('K') || trimmed.starts_with('L'))
            && trimmed.len() >= 51
            && trimmed.len() <= 52
        {
            return Ok(KeyFormat::WIF);
        }

        // Hex: 64 hex chars
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(KeyFormat::Hex);
        }

        // BIP39: 12/15/18/21/24 space-separated words
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if matches!(words.len(), 12 | 15 | 18 | 21 | 24)
            && words.iter().all(|w| w.chars().all(|c| c.is_ascii_lowercase()))
        {
            return Ok(KeyFormat::BIP39);
        }

        // Fallback: treat as brainwallet passphrase
        if !trimmed.is_empty() {
            return Ok(KeyFormat::Brainwallet);
        }

        Err(anyhow::anyhow!(
            "Could not detect key format. Expected: 64 hex chars, WIF, BIP39 mnemonic, or brainwallet text"
        ))
    }

    /// Parse private key from various formats into 32-byte scalar(s)
    pub fn parse_private_key(
        input: &str,
        format: KeyFormat,
        passphrase: Option<&str>,
        _derivation_path: Option<&str>,
    ) -> Result<Vec<[u8; 32]>> {
        let network = Network::Bitcoin;
        let secp = Secp256k1::<All>::new();

        match format {
            KeyFormat::Hex => {
                let bytes = hex::decode(input.trim())?;
                if bytes.len() != 32 {
                    return Err(anyhow::anyhow!(
                        "Hex key must be exactly 64 characters (32 bytes)"
                    ));
                }
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                Ok(vec![key])
            }

            KeyFormat::WIF => {
                let pk = PrivateKey::from_str(input.trim())?;
                Ok(vec![pk.inner.secret_bytes()])
            }

            KeyFormat::BIP39 => {
                let mnemonic = Mnemonic::parse(input.trim())
                    .map_err(|e| anyhow::anyhow!("Invalid BIP39 mnemonic: {}", e))?;
                let pass = passphrase.unwrap_or("");
                let seed_bytes: [u8; 64] = mnemonic.to_seed(pass);

                let xpriv = bitcoin::bip32::Xpriv::new_master(network, &seed_bytes)
                    .map_err(|e| anyhow::anyhow!("Failed to create master key: {}", e))?;
                let path = bitcoin::bip32::DerivationPath::from_str("m/44'/0'/0'/0/0")
                    .map_err(|e| anyhow::anyhow!("Invalid derivation path: {}", e))?;
                let derived = xpriv
                    .derive_priv(&secp, &path)
                    .map_err(|e| anyhow::anyhow!("Derivation failed: {}", e))?;

                Ok(vec![derived.private_key.secret_bytes()])
            }

            KeyFormat::Brainwallet => {
                let hash = Sha256::digest(input.trim().as_bytes());
                let key: [u8; 32] = hash.into();
                Ok(vec![key])
            }
        }
    }

    /// Check a single private key against the UTXO index
    pub async fn check_key(
        index: &FlatIndex,
        privkey_bytes: [u8; 32],
        input_display: String,
        input_format: String,
    ) -> KeyCheckResult {
        let secp = Secp256k1::<All>::new();
        let network = Network::Bitcoin;

        let secret_key = match bitcoin::secp256k1::SecretKey::from_slice(&privkey_bytes) {
            Ok(sk) => sk,
            Err(e) => {
                return KeyCheckResult {
                    input: input_display,
                    input_format,
                    privkey_hex: hex::encode(privkey_bytes),
                    pubkey_hex: String::new(),
                    addresses: KeyAddresses {
                        legacy: "error".into(),
                        segwit: "error".into(),
                        wrapped: "error".into(),
                        taproot: "error".into(),
                    },
                    matches: Vec::new(),
                    total_balance_sats: 0,
                    total_balance_btc: 0.0,
                    error: Some(format!("Invalid private key: {}", e)),
                };
            }
        };

        let pk = PrivateKey {
            inner: secret_key,
            network: network.into(),
            compressed: true,
        };
        let pubkey = pk.public_key(&secp);
        let compressed = match CompressedPublicKey::from_private_key(&secp, &pk) {
            Ok(c) => c,
            Err(e) => {
                return KeyCheckResult {
                    input: input_display,
                    input_format,
                    privkey_hex: hex::encode(privkey_bytes),
                    pubkey_hex: hex::encode(pubkey.to_bytes()),
                    addresses: KeyAddresses {
                        legacy: "error".into(),
                        segwit: "error".into(),
                        wrapped: "error".into(),
                        taproot: "error".into(),
                    },
                    matches: Vec::new(),
                    total_balance_sats: 0,
                    total_balance_btc: 0.0,
                    error: Some(format!("Failed to compress pubkey: {}", e)),
                };
            }
        };
        let xonly: UntweakedPublicKey = compressed.into();

        let mut matches = Vec::new();
        let mut total_balance = 0u64;

        // Legacy P2PKH
        let legacy_addr = Address::p2pkh(&pubkey, network);
        let legacy_script = legacy_addr.script_pubkey();
        let val = index.lookup(legacy_script.as_bytes());
        if val > 0 {
            total_balance += val;
            matches.push(UTXOMatch {
                address: legacy_addr.to_string(),
                address_type: "P2PKH (Legacy)".to_string(),
                value_sats: val,
                value_btc: val as f64 / 1e8,
                script_hex: hex::encode(legacy_script.as_bytes()),
            });
        }

        // Native Segwit P2WPKH
        let segwit_addr = Address::p2wpkh(&compressed, network);
        let segwit_script = segwit_addr.script_pubkey();
        let val = index.lookup(segwit_script.as_bytes());
        if val > 0 {
            total_balance += val;
            matches.push(UTXOMatch {
                address: segwit_addr.to_string(),
                address_type: "P2WPKH (Segwit)".to_string(),
                value_sats: val,
                value_btc: val as f64 / 1e8,
                script_hex: hex::encode(segwit_script.as_bytes()),
            });
        }

        // Wrapped Segwit P2SH-P2WPKH
        let wrapped_addr = Address::p2shwpkh(&compressed, network);
        let wrapped_script = wrapped_addr.script_pubkey();
        let val = index.lookup(wrapped_script.as_bytes());
        if val > 0 {
            total_balance += val;
            matches.push(UTXOMatch {
                address: wrapped_addr.to_string(),
                address_type: "P2SH-P2WPKH (Wrapped)".to_string(),
                value_sats: val,
                value_btc: val as f64 / 1e8,
                script_hex: hex::encode(wrapped_script.as_bytes()),
            });
        }

        // Taproot P2TR
        let taproot_addr = Address::p2tr(&secp, xonly, None, network);
        let taproot_script = taproot_addr.script_pubkey();
        let val = index.lookup(taproot_script.as_bytes());
        if val > 0 {
            total_balance += val;
            matches.push(UTXOMatch {
                address: taproot_addr.to_string(),
                address_type: "P2TR (Taproot)".to_string(),
                value_sats: val,
                value_btc: val as f64 / 1e8,
                script_hex: hex::encode(taproot_script.as_bytes()),
            });
        }

        KeyCheckResult {
            input: input_display,
            input_format,
            privkey_hex: hex::encode(privkey_bytes),
            pubkey_hex: hex::encode(pubkey.to_bytes()),
            addresses: KeyAddresses {
                legacy: legacy_addr.to_string(),
                segwit: segwit_addr.to_string(),
                wrapped: wrapped_addr.to_string(),
                taproot: taproot_addr.to_string(),
            },
            matches,
            total_balance_sats: total_balance,
            total_balance_btc: total_balance as f64 / 1e8,
            error: None,
        }
    }
}
