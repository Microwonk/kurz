use anyhow::{Result, bail};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortUrl {
    pub id: i64,
    pub slug: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
    pub hit_count: i64,
}

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

macro_rules! connect {
    ($self:ident) => {{
        let Ok(conn) = $self.conn.lock() else {
            bail!("Mutex lock poisoned!");
        };
        conn
    }};
}

impl Db {
    /// opens connection to database and creates the
    /// necessary `short_urls` table
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS short_urls (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                slug        TEXT    NOT NULL UNIQUE,
                original_url TEXT   NOT NULL,
                created_at  TEXT    NOT NULL,
                hit_count   INTEGER NOT NULL DEFAULT 0
            );",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert_url(&self, slug: &str, original_url: &str) -> Result<ShortUrl> {
        let conn = connect!(self);

        let now = Utc::now();
        conn.execute(
            "INSERT INTO short_urls (slug, original_url, created_at, hit_count) VALUES (?1, ?2, ?3, 0)",
            params![slug, original_url, now.to_rfc3339()],
        )?;

        let id = conn.last_insert_rowid();

        Ok(ShortUrl {
            id,
            slug: slug.to_string(),
            original_url: original_url.to_string(),
            created_at: now,
            hit_count: 0,
        })
    }

    pub fn get_url_by_slug(&self, slug: &str) -> Result<Option<ShortUrl>> {
        let conn = connect!(self);

        let mut stmt = conn.prepare(
            "SELECT id, slug, original_url, created_at, hit_count FROM short_urls WHERE slug = ?1",
        )?;

        let mut rows = stmt.query(params![slug])?;

        if let Some(row) = rows.next()? {
            let created_at: String = row.get(3)?;

            Ok(Some(ShortUrl {
                id: row.get(0)?,
                slug: row.get(1)?,
                original_url: row.get(2)?,
                created_at: created_at
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
                hit_count: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn url_exists(&self, slug: &str) -> Result<bool> {
        let conn = connect!(self);

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM short_urls WHERE slug = ?1",
            params![slug],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    pub fn increment_url_hits(&self, slug: &str) -> Result<()> {
        let conn = connect!(self);

        conn.execute(
            "UPDATE short_urls SET hit_count = hit_count + 1 WHERE slug = ?1",
            params![slug],
        )?;

        Ok(())
    }

    pub fn list_all_urls(&self) -> Result<Vec<ShortUrl>> {
        let conn = connect!(self);

        let mut stmt = conn.prepare(
            "SELECT id, slug, original_url, created_at, hit_count FROM short_urls ORDER BY id DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let created_at: String = row.get(3)?;
            Ok(ShortUrl {
                id: row.get(0)?,
                slug: row.get(1)?,
                original_url: row.get(2)?,
                created_at: created_at
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
                hit_count: row.get(4)?,
            })
        })?;

        rows.map(|e| e.map_err(Into::into)).collect()
    }

    pub fn delete_url_by_slug(&self, slug: &str) -> Result<bool> {
        let conn = connect!(self);
        let affected = conn.execute("DELETE FROM short_urls WHERE slug = ?1", params![slug])?;
        Ok(affected > 0)
    }
}
