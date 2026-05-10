//! config.rs — Configuration management for HerbReady.
//!
//! Reads/writes:
//!   * config.ini      — database connection settings  (INI format)
//!   * app_config.json — drug and department lists     (JSON)
//!
//! Files are stored in the OS local-data directory:
//!   Windows : %LOCALAPPDATA%\HerbReady\
//!   macOS   : ~/Library/Application Support/HerbReady/
//!   Linux   : ~/.local/share/HerbReady/

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::crypto;

// ---------------------------------------------------------------------------
// DatabaseConfig
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "127.0.0.1".into(),
            port: 3306,
            name: "hosxp".into(),
            user: "root".into(),
            password: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// AppConfig (app_config.json)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DrugConfig {
    pub icode: String,
    pub abbr: String,
    pub course_days: i32,
    pub capsules: i32,
    pub drug_name: String,
    /// Whether this drug is enabled for display. New field — default true
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeptConfig {
    pub code: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LabRuleConfig {
    pub lab_items_code: String,
    pub lab_items_name: String,
    /// Numeric threshold for comparison
    pub threshold: f64,
    /// Alert when result > threshold
    #[serde(default)]
    pub compare_gt: bool,
    /// Alert when result == threshold (within ±0.001)
    #[serde(default)]
    pub compare_eq: bool,
    /// Alert when result < threshold
    #[serde(default)]
    pub compare_lt: bool,
}

/// One herb drug entry in an interaction rule.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HerbDrugEntry {
    pub icode: String,
    pub name: String,
}

/// One Herb/Drug interaction rule: a modern (conventional) drug that interacts
/// with one or more herb drugs.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HerbDrugInteraction {
    pub modern_drug_icode: String,
    pub modern_drug_name: String,
    pub herb_drugs: Vec<HerbDrugEntry>,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub drugs: Vec<DrugConfig>,
    pub departments: Vec<DeptConfig>,
    #[serde(default)]
    pub lab_rules: Vec<LabRuleConfig>,
    #[serde(default)]
    pub herb_drug_interactions: Vec<HerbDrugInteraction>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            drugs: vec![],
            departments: vec![DeptConfig {
                code: "011".into(),
                name: "แพทย์แผนไทย".into(),
            }],
            lab_rules: vec![],
            herb_drug_interactions: vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// Path resolution — AppData / local-data directory
// ---------------------------------------------------------------------------

/// Return (and create if needed) the HerbReady config directory inside the
/// OS local-data folder.
fn config_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("ไม่สามารถหา AppData directory"))?
        .join("HerbReady");
    fs::create_dir_all(&dir)
        .with_context(|| format!("ไม่สามารถสร้าง directory {}", dir.display()))?;
    Ok(dir)
}

/// Full path to config.ini inside AppData/HerbReady/.
fn config_ini_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.ini"))
}

/// Full path to app_config.json inside AppData/HerbReady/.
fn app_config_json_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("app_config.json"))
}

// ---------------------------------------------------------------------------
// Read / Write config.ini
// ---------------------------------------------------------------------------

/// Parse a minimal INI file.  Returns a flat map of "section.key" → value.
fn parse_ini(content: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut current_section = String::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].trim().to_lowercase();
        } else if let Some(eq_pos) = line.find('=') {
            if line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            let key = line[..eq_pos].trim().to_lowercase();
            let val = line[eq_pos + 1..].trim().to_string();
            map.insert(format!("{}.{}", current_section, key), val);
        }
    }
    map
}

/// Internal struct for serializing all DB fields into one encrypted blob.
#[derive(Serialize, Deserialize)]
struct DbConnectionBlob {
    host: String,
    port: u16,
    name: String,
    user: String,
    password: String,
}

/// Master key source — derived from machine-specific data.
/// In production, consider deriving from OS keychain/credential store.
fn get_master_key() -> Result<String> {
    let machine_id = whoami::fallible::devicename()
        .unwrap_or_else(|_| "HerbReady-DefaultMachine".to_string());
    let username = whoami::username();
    Ok(format!("HerbReady-v2-MasterKey-{}-{}", username, machine_id))
}

/// Read database config from config.ini.
/// Returns `DatabaseConfig::default()` when the file does not exist yet.
pub fn read_db_config() -> Result<DatabaseConfig> {
    let path = config_ini_path()?;

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DatabaseConfig::default());
        }
        Err(e) => {
            return Err(e)
                .with_context(|| format!("ไม่สามารถอ่านไฟล์ config.ini ที่ {}", path.display()));
        }
    };

    let ini = parse_ini(&content);

    let master_key = get_master_key()?;

    let blob: DbConnectionBlob = if let Some(encrypted) = ini.get("database.connection") {
        let json = crypto::decrypt(encrypted, &master_key)?;
        serde_json::from_str(&json).context("ไม่สามารถแปลงข้อมูล connection")?
    } else {
        DbConnectionBlob {
            host: ini.get("database.host").cloned().unwrap_or_else(|| "127.0.0.1".into()),
            port: ini.get("database.port").and_then(|v| v.parse().ok()).unwrap_or(3306),
            name: ini.get("database.name").cloned().unwrap_or_else(|| "hosxp".into()),
            user: ini.get("database.user").cloned().unwrap_or_else(|| "root".into()),
            password: ini.get("database.password").cloned().unwrap_or_default(),
        }
    };

    Ok(DatabaseConfig {
        host: blob.host,
        port: blob.port,
        name: blob.name,
        user: blob.user,
        password: blob.password,
    })
}

/// Write database config to config.ini.
pub fn write_db_config(cfg: &DatabaseConfig) -> Result<()> {
    let path = config_ini_path()?;

    let master_key = get_master_key()?;

    let blob = DbConnectionBlob {
        host: cfg.host.clone(),
        port: cfg.port,
        name: cfg.name.clone(),
        user: cfg.user.clone(),
        password: cfg.password.clone(),
    };
    let json = serde_json::to_string(&blob).context("ไม่สามารถแปลง DbConnectionBlob เป็น JSON")?;
    let encrypted_connection = crypto::encrypt(&json, &master_key)?;

    let content = format!(
        "[database]\nconnection = {}\n\n[app]\ntitle        = HerbReady - ระบบจ่ายยาสมุนไพร\ndefault_dept = 011\ntheme        = dark\n",
        encrypted_connection
    );
    fs::write(&path, content)
        .with_context(|| format!("ไม่สามารถเขียน config.ini ที่ {}", path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Read / Write app_config.json
// ---------------------------------------------------------------------------

/// Read app config (drugs + departments) from app_config.json.
/// Returns `AppConfig::default()` when the file does not exist yet.
pub fn read_app_config() -> Result<AppConfig> {
    let path = app_config_json_path()?;

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(AppConfig::default());
        }
        Err(e) => {
            return Err(e)
                .with_context(|| format!("ไม่สามารถอ่านไฟล์ app_config.json ที่ {}", path.display()));
        }
    };

    let cfg: AppConfig =
        serde_json::from_str(&content).context("ไม่สามารถแปลงข้อมูล app_config.json")?;
    Ok(cfg)
}

/// Write app config to app_config.json.
pub fn write_app_config(cfg: &AppConfig) -> Result<()> {
    let path = app_config_json_path()?;
    let content = serde_json::to_string_pretty(cfg).context("ไม่สามารถแปลง AppConfig เป็น JSON")?;
    fs::write(&path, content)
        .with_context(|| format!("ไม่สามารถเขียน app_config.json ที่ {}", path.display()))?;
    Ok(())
}
