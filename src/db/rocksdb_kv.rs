use db;

extern crate rocksdb;
extern crate tempdir;

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

    fn put(&self, key: String, value: String) -> Result<(), String> {
        use self::rocksdb::Writable;
        self.rocks_db.put(key.as_bytes(), value.as_bytes())
    }

    fn get(&self, key: String) -> Result<Option<String>, String> {
        self.rocks_db.get(key.as_bytes()).map(|ob| ob.and_then(|b| b.to_utf8().map(|x| x.to_owned())))
    }

    fn get_prefix(&self, prefix: String) -> Vec<(String, String)> {
        use self::rocksdb::{Direction, IteratorMode};
        self.rocks_db.iterator(IteratorMode::From(prefix.as_bytes(), Direction::Forward))
            .map(|p| (String::from_utf8(p.0.into_vec()).unwrap(), String::from_utf8(p.1.into_vec()).unwrap()))
            .take_while(|p| p.0.starts_with(&prefix))
            .collect::<Vec<(String, String)>>()
    }
}

impl db::KV<String, String> for RocksDBKV {
    fn put(&self, key: String, value: String) -> Result<(), String> {
        unimplemented!();
    }

    fn get(&self, key: String) -> Result<Option<String>, String> {
        unimplemented!();
    }
}

#[test]
fn get_nonexistent_key_test() {
    if let Ok(dir) = tempdir::TempDir::new("get_nonexistent_key_test") {
        let path = dir.path().join("db");

        let db = RocksDBKV::new(path.as_path());

        assert_eq!(db.get("nonexistent key".to_owned()), Ok(None))
    }
}

#[test]
fn put_and_get_key_test() {
    if let Ok(dir) = tempdir::TempDir::new("put_and_get_key_test") {
        let path = dir.path().join("db");

        let db = RocksDBKV::new(path.as_path());

        assert_eq!(db.get("key".to_owned()), Ok(None));
        assert_eq!(db.put("key".to_owned(), "value".to_owned()), Ok(()));
        assert_eq!(db.get("key".to_owned()), Ok(Some("value".to_owned())))
    }
}

#[test]
fn overwrite_key_test() {
    if let Ok(dir) = tempdir::TempDir::new("overwrite_key_test") {
        let path = dir.path().join("db");

        let db = RocksDBKV::new(path.as_path());

        assert_eq!(db.put("key".to_owned(), "value1".to_owned()), Ok(()));
        assert_eq!(db.get("key".to_owned()), Ok(Some("value1".to_owned())));
        assert_eq!(db.put("key".to_owned(), "value2".to_owned()), Ok(()));
        assert_eq!(db.get("key".to_owned()), Ok(Some("value2".to_owned())))
    }
}

#[test]
fn fetch_keys_by_prefix_test() {
    if let Ok(dir) = tempdir::TempDir::new("fetch_keys_by_prefix_test") {
        let path = dir.path().join("db");

        let db = RocksDBKV::new(path.as_path());

        assert_eq!(db.put("a_key".to_owned(), "not-in-set1".to_owned()), Ok(()));
        assert_eq!(db.put("key1".to_owned(), "value1".to_owned()), Ok(()));
        assert_eq!(db.put("key2".to_owned(), "value2".to_owned()), Ok(()));
        assert_eq!(db.put("z_key".to_owned(), "not-in-set2".to_owned()), Ok(()));

        assert_eq!(
            db.get_prefix("key".to_owned()),
            vec![("key1".to_owned(), "value1".to_owned()), ("key2".to_owned(), "value2".to_owned())])
    }
}
