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
fn put_and_get_key_test() {
    if let Ok(dir) = tempdir::TempDir::new("put_and_get_key_test") {
        let path = dir.path().join("db");

        let db = RocksDBKV::new(path.as_path());

        assert_eq!(db.put("key".to_owned(), "value".to_owned()), Ok(()));
        assert_eq!(db.get("key".to_owned()), Ok(Some("value".to_owned())))
    }
}
