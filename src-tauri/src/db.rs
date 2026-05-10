//! db.rs — Global MySQL connection pool for HerbReady.
//!
//! Uses `sqlx::MySqlPool` stored in a `once_cell::sync::OnceCell` so it
//! can be initialised once at startup and shared across async command handlers.

use std::time::Duration;

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// Global pool singleton
// ---------------------------------------------------------------------------

static DB_POOL: OnceCell<MySqlPool> = OnceCell::new();

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Initialise the global pool.  Calling this more than once is an error —
/// callers should call `reset_pool` first if they want to reconfigure.
pub async fn init_pool(
  host: &str,
  port: u16,
  dbname: &str,
  user: &str,
  password: &str,
) -> Result<()> {
  let url = build_url(host, port, dbname, user, password);
  let pool = MySqlPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(Duration::from_secs(10))
    .connect(&url)
    .await
    .with_context(|| format!("ไม่สามารถเชื่อมต่อฐานข้อมูล: {host}:{port}/{dbname}"))?;

  // If already set, just verify connectivity (pool was already initialised).
  if DB_POOL.get().is_some() {
    // Re-use existing pool — test connectivity only.
    return Ok(());
  }

  DB_POOL
    .set(pool)
    .map_err(|_| anyhow::anyhow!("Pool already initialised"))?;
  Ok(())
}

/// Replace the global pool (used when db settings change).
///
/// Closes the old pool and installs a new one.  Because `OnceCell` does not
/// support mutation we use an `UnsafeCell`-backed trick via a dedicated
/// `Mutex<Option<MySqlPool>>` stored separately.
///
/// For simplicity in this codebase we maintain a second `Mutex` alongside
/// the `OnceCell`.
pub async fn reset_pool(
  host: &str,
  port: u16,
  dbname: &str,
  user: &str,
  password: &str,
) -> Result<()> {
  let url = build_url(host, port, dbname, user, password);
  let pool = MySqlPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(Duration::from_secs(10))
    .connect(&url)
    .await
    .with_context(|| format!("ไม่สามารถเชื่อมต่อฐานข้อมูล: {host}:{port}/{dbname}"))?;

  // Replace the mutable pool
  let mut guard = MUTABLE_POOL.lock().await;
  *guard = Some(pool);
  Ok(())
}

/// Return a reference to the active pool (either the mutable override or the
/// original OnceCell pool).
pub async fn get_pool() -> Result<MySqlPool> {
  // Prefer the mutable pool (set via reset_pool / cmd_connect_db).
  {
    let guard = MUTABLE_POOL.lock().await;
    if let Some(pool) = guard.as_ref() {
      return Ok(pool.clone());
    }
  }
  // Fall back to the OnceCell pool.
  DB_POOL
    .get()
    .cloned()
    .ok_or_else(|| anyhow::anyhow!("ยังไม่ได้เชื่อมต่อฐานข้อมูล — กรุณาตั้งค่าการเชื่อมต่อก่อน"))
}

/// Open a throw-away single connection and return the server version string.
/// Does NOT install a pool.
pub async fn test_connection(
  host: &str,
  port: u16,
  dbname: &str,
  user: &str,
  password: &str,
) -> Result<String> {
  let url = build_url(host, port, dbname, user, password);
  let pool = MySqlPoolOptions::new()
    .max_connections(1)
    .acquire_timeout(Duration::from_secs(10))
    .connect(&url)
    .await
    .with_context(|| format!("เชื่อมต่อไม่ได้: {host}:{port}/{dbname}"))?;

  let row: (String,) = sqlx::query_as("SELECT VERSION()")
    .fetch_one(&pool)
    .await
    .context("ไม่สามารถอ่าน VERSION() จากเซิร์ฟเวอร์")?;

  pool.close().await;
  Ok(format!("เชื่อมต่อสำเร็จ — MySQL {}", row.0))
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Mutable pool (replaces the OnceCell pool when db settings change).
static MUTABLE_POOL: tokio::sync::Mutex<Option<MySqlPool>> = tokio::sync::Mutex::const_new(None);

fn build_url(host: &str, port: u16, dbname: &str, user: &str, password: &str) -> String {
  // Percent-encode the password in case it contains special characters
  let pw_encoded = url_encode(password);
  format!(
    "mysql://{}:{}@{}:{}/{}?charset=utf8mb4",
    user, pw_encoded, host, port, dbname
  )
}

/// Minimal URL percent-encoder for the password field.
fn url_encode(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  for b in s.bytes() {
    match b {
      b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
      _ => {
        out.push('%');
        out.push_str(&format!("{:02X}", b));
      }
    }
  }
  out
}
