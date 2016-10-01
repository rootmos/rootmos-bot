use db;
use std::collections::HashMap;

pub struct HashMapKV {
    inner: HashMap<String, String>
}

impl HashMapKV {
    pub fn new() -> HashMapKV {
        HashMapKV { inner: HashMap::new() }
    }
}

impl db::KV<String, String> for HashMapKV {
    fn put(&mut self, key: &String, value: &String) -> Result<(), String> {
        let _ = self.inner.insert(key.clone(), value.clone());
        Ok(())
    }

    fn get(&self, key: &String) -> Result<Option<String>, String> {
        Ok(self.inner.get(key).map(|v| v.clone()))
    }

    fn get_prefix(&self, prefix: &String) -> Vec<(String, String)> {
        self.inner.iter()
           .filter(|&(k,_)| k.starts_with(prefix))
           .map(|(k, v)| (k.clone(), v.clone()))
           .collect::<Vec<(String, String)>>()
    }

    fn remove(&mut self, key: &String) -> Result<(), String> {
        self.inner.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use db;
    use db::hashmap_kv::*;

    #[test]
    fn get_nonexistent_key_test() {
        let db = HashMapKV::new();
        db::test::get_nonexistent_key_test(db, rand_key())
    }

    #[test]
    fn put_and_get_key_test() {
        let db = HashMapKV::new();
        db::test::put_and_get_key_test(db, rand_key(), rand_value())
    }

    #[test]
    fn overwrite_key_test() {
        let db = HashMapKV::new();
        db::test::overwrite_key_test(db, rand_key(), rand_value(), rand_value())
    }

    #[test]
    fn remove_key_test() {
        let db = HashMapKV::new();
        db::test::remove_key_test(db, rand_key(), rand_value())
    }

    #[test]
    fn fetch_keys_by_prefix_test() {
        let db = HashMapKV::new();
        let prefix = "prefix".to_owned();
        db::test::fetch_keys_by_prefix_test(db, prefix, rand_key, prefixed_key, rand_value)
    }

    extern crate rand;

    fn rand_key() -> String {
        use self::rand::Rng;
        let salt: String = rand::thread_rng().gen_ascii_chars().take(5).collect();
        format!("key{}", salt)
    }

    fn rand_value() -> String {
        use self::rand::Rng;
        let salt: String = rand::thread_rng().gen_ascii_chars().take(5).collect();
        format!("value{}", salt)
    }

    fn prefixed_key() -> String {
        format!("prefix-{}", rand_key())
    }
}
