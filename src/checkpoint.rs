//! Autosave and restore of distinguished-point progress.
//!
//! Saves the DPTable entries (affine X, distance, type) to a JSON file
//! every N seconds so the solver can resume after a restart.

use crate::cpu::DPTable;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct CheckpointEntry {
    ax: String, // affine_x hex (32 bytes BE)
    di: String, // dist hex (32 bytes LE)
    kt: u32,    // ktype: 0=tame 1=wild1 2=wild2
}

/// Checkpoint file written to disk every checkpoint interval.
#[derive(Serialize, Deserialize)]
pub struct CheckpointData {
    pub version: String,
    /// Compressed pubkey hex used to validate on reload.
    pub pubkey_hex: String,
    /// Search range start hex used to validate on reload.
    pub start_hex: String,
    pub range_bits: u32,
    pub saved_at: u64,
    pub total_ops: u64,
    entries: Vec<CheckpointEntry>,
}

impl CheckpointData {
    pub fn new(
        pubkey_hex: &str,
        start_hex: &str,
        range_bits: u32,
        total_ops: u64,
        dp_table: &DPTable,
    ) -> Self {
        let entries = dp_table
            .export_entries()
            .into_iter()
            .map(|(ax, di, kt)| CheckpointEntry {
                ax: hex::encode(ax),
                di: hex::encode(di),
                kt,
            })
            .collect();

        let saved_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            pubkey_hex: pubkey_hex.to_lowercase(),
            start_hex: start_hex.to_lowercase(),
            range_bits,
            saved_at,
            total_ops,
            entries,
        }
    }

    /// Returns true if the checkpoint was created for the same search parameters.
    pub fn matches(&self, pubkey_hex: &str, start_hex: &str, range_bits: u32) -> bool {
        self.pubkey_hex == pubkey_hex.to_lowercase()
            && self.start_hex == start_hex.to_lowercase()
            && self.range_bits == range_bits
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Import all stored DPs into `dp_table` (no collision check — we already
    /// know this run hasn't found the key yet).
    pub fn restore_into(&self, dp_table: &mut DPTable) -> usize {
        let mut count = 0;
        for e in &self.entries {
            let Ok(ax_vec) = hex::decode(&e.ax) else { continue };
            let Ok(di_vec) = hex::decode(&e.di) else { continue };
            if ax_vec.len() != 32 || di_vec.len() != 32 {
                continue;
            }
            let mut ax = [0u8; 32];
            let mut di = [0u8; 32];
            ax.copy_from_slice(&ax_vec);
            di.copy_from_slice(&di_vec);
            dp_table.import_entry(ax, di, e.kt);
            count += 1;
        }
        count
    }
}

/// Atomically write the checkpoint (temp file → rename) so a crash during
/// save never corrupts the previous good checkpoint.
pub fn save_checkpoint(path: &Path, data: &CheckpointData) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn load_checkpoint(path: &Path) -> Result<CheckpointData> {
    let json = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str::<CheckpointData>(&json)?)
}
