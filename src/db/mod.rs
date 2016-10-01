pub trait KV<K, V> {
    fn put(&self, key: &K, value: &V) -> Result<(), String>;
    fn get(&self, key: &K) -> Result<Option<V>, String>;
    fn get_prefix(&self, key: &K) -> Vec<(K, V)>;
    fn remove(&self, key: &K) -> Result<(), String>;
}

pub mod rocksdb_kv;

#[cfg(test)]
pub mod test;
