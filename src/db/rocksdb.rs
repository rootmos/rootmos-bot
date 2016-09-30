use db;

pub struct RocksDB {
    rocks_db: (),
}

impl RocksDB {
    pub fn new(file: String) -> RocksDB {
        unimplemented!();
    }
}

impl db::KV<String, String> for RocksDB {
    fn put(&self, key: String, value: String) -> Result<(), String> {
        unimplemented!();
    }

    fn get(&self, key: String) -> Result<Option<String>, String> {
        unimplemented!();
    }
}
