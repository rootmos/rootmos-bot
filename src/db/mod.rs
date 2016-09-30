trait KV<K, V> {
    fn put(&self, key: K, value: V) -> Result<(), String>;
    fn get(&self, key: K) -> Result<Option<V>, String>;
}

pub mod rocksdb_kv;
