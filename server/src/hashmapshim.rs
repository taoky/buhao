use std::path::Path;

use buhao_lib::InodeId;

/// A shim over std hashmap or sqlite3
pub trait HashMapShim<K, V>: Send {
    fn insert(&mut self, key: K, value: V);
    fn get(&self, key: &K) -> Option<V>;
    fn remove(&mut self, key: &K);
    fn values(&self) -> Vec<V>;
}

#[derive(Debug)]
pub struct StdHashMap<K, V> {
    map: std::collections::HashMap<K, V>,
}

impl<K, V> HashMapShim<K, V> for StdHashMap<K, V>
where
    K: std::hash::Hash + Eq + Send,
    V: Clone + Send,
{
    fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    fn get(&self, key: &K) -> Option<V> {
        self.map.get(key).cloned()
    }

    fn remove(&mut self, key: &K) {
        self.map.remove(key);
    }

    fn values(&self) -> Vec<V> {
        self.map.values().cloned().collect()
    }
}

impl<K, V> StdHashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct SqliteHashMap<K, V> {
    pub conn: rusqlite::Connection,
    key_type: std::marker::PhantomData<K>,
    value_type: std::marker::PhantomData<V>,
}

impl<V> SqliteHashMap<InodeId, V> {
    pub fn create(&self) -> rusqlite::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS inodemap (
                key INT PRIMARY KEY,
                value TEXT,
                epoch INT
            )",
            (),
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_epoch on inodemap(epoch)",
            (),
        )?;
        Ok(())
    }

    pub fn drop_(&self) -> rusqlite::Result<()> {
        self.conn.execute("DROP TABLE IF EXISTS inodemap", ())?;
        Ok(())
    }

    pub fn new(path: &Path) -> rusqlite::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        let obj = Self {
            conn,
            key_type: std::marker::PhantomData,
            value_type: std::marker::PhantomData,
        };
        obj.create()?;
        Ok(obj)
    }
}

impl<V> HashMapShim<InodeId, V> for SqliteHashMap<InodeId, V>
where
    V: serde::Serialize + serde::de::DeserializeOwned + Send,
{
    fn insert(&mut self, key: InodeId, value: V) {
        let value = serde_json::to_string(&value).unwrap();
        self.conn
            .execute(
                "INSERT OR REPLACE INTO inodemap (key, value, epoch) VALUES (?1, ?2, ?3)",
                rusqlite::params![key, value, 0],
            )
            .unwrap();
    }

    fn get(&self, key: &InodeId) -> Option<V> {
        let value: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM inodemap WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get(0),
            )
            .ok();
        value.and_then(|value| serde_json::from_str(&value).ok())
    }

    fn remove(&mut self, key: &InodeId) {
        self.conn
            .execute(
                "DELETE FROM inodemap WHERE key = ?1",
                rusqlite::params![key],
            )
            .unwrap();
    }

    fn values(&self) -> Vec<V> {
        let mut stmt = self.conn.prepare("SELECT value FROM inodemap").unwrap();
        let rows = stmt.query_map([], |row| row.get(0)).unwrap();
        rows.map(|row| row.unwrap())
            .map(|value: String| serde_json::from_str(&value).unwrap())
            .collect()
    }
}
