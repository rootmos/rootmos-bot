pub trait KV<K, V> {
    fn put(&mut self, key: &K, value: &V) -> Result<(), String>;
    fn get(&self, key: &K) -> Result<Option<V>, String>;
    fn get_prefix(&self, key: &K) -> Vec<(K, V)>;
    fn remove(&mut self, key: &K) -> Result<(), String>;
}

pub mod rocksdb_kv;
pub mod hashmap_kv;

#[cfg(test)]
pub mod test;
