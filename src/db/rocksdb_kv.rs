extern crate rocksdb;

use db;
use std::path::Path;

pub struct RocksDBKV {
    rocks_db: rocksdb::DB,
}

impl RocksDBKV {
    pub fn new(path: &Path) -> RocksDBKV {
        let path_str = path.to_str().unwrap();
        let db = rocksdb::DB::open_default(path_str).unwrap();
        RocksDBKV { rocks_db: db }
    }

    fn put(&self, key: &String, value: &String) -> Result<(), String> {
        use self::rocksdb::Writable;
        self.rocks_db.put(key.as_bytes(), value.as_bytes())
    }

    fn get(&self, key: &String) -> Result<Option<String>, String> {
        self.rocks_db.get(key.as_bytes()).map(|ob| ob.and_then(|b| b.to_utf8().map(|x| x.to_owned())))
    }

    fn get_prefix(&self, prefix: &String) -> Vec<(String, String)> {
        use self::rocksdb::{Direction, IteratorMode};
        self.rocks_db.iterator(IteratorMode::From(prefix.as_bytes(), Direction::Forward))
            .map(|p| (String::from_utf8(p.0.into_vec()).unwrap(), String::from_utf8(p.1.into_vec()).unwrap()))
            .take_while(|p| p.0.starts_with(prefix))
            .collect::<Vec<(String, String)>>()
    }
}

impl db::KV<String, String> for RocksDBKV {
    fn put(&self, key: &String, value: &String) -> Result<(), String> { self.put(key, value) }
    fn get(&self, key: &String) -> Result<Option<String>, String> { self.get(key) }
    fn get_prefix(&self, prefix: &String) -> Vec<(String, String)> { self.get_prefix(prefix) }
}


#[cfg(test)]
mod test {
    use db;
    use db::rocksdb_kv;

    #[test]
    fn get_nonexistent_key_test() {
        let db = new_temp_db();
        db::test::get_nonexistent_key_test(db, rand_key())
    }

    #[test]
    fn put_and_get_key_test() {
        let db = new_temp_db();
        db::test::put_and_get_key_test(db, rand_key(), rand_value())
    }

    #[test]
    fn overwrite_key_test() {
        let db = new_temp_db();
        db::test::overwrite_key_test(db, rand_key(), rand_value(), rand_value())
    }

    #[test]
    fn fetch_keys_by_prefix_test() {
        let db = new_temp_db();
        let prefix = "prefix".to_owned();
        db::test::fetch_keys_by_prefix_test(db, prefix, rand_key, prefixed_key, rand_value)
    }


    fn new_temp_db() -> rocksdb_kv::RocksDBKV {
        extern crate tempdir;
        let path = tempdir::TempDir::new("rocksdb_kv_test").unwrap().path().join("db");
        rocksdb_kv::RocksDBKV::new(path.as_path())
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
