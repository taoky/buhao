use buhao_lib::InodeId;

/// A shim over std hashmap or sqlite3
trait HashMapShim<K, V> {
    fn insert(&mut self, key: K, value: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K);
}

#[derive(Debug)]
pub struct StdHashMap<K, V> {
    map: std::collections::HashMap<K, V>,
}

impl<K, V> HashMapShim<K, V> for StdHashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    fn remove(&mut self, key: &K) {
        self.map.remove(key);
    }
}

impl<K, V> StdHashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct SqliteHashMap<K, V> {
    conn: rusqlite::Connection,
    key_type: std::marker::PhantomData<K>,
    value_type: std::marker::PhantomData<V>,
}

impl<V> SqliteHashMap<InodeId, V> {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS inodemap (
                key INT PRIMARY KEY,
                value BLOB,
                epoch INT
            )",
            (),
        )?;
        Ok(Self {
            conn,
            key_type: std::marker::PhantomData,
            value_type: std::marker::PhantomData,
        })
    }
}