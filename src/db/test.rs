use db;
use std::fmt::Debug;

pub fn get_nonexistent_key_test<K: Eq + Debug, V: Eq + Debug, T: db::KV<K, V>>(db: T, k: K) {
    assert_eq!(db.get(&k), Ok(None))
}

pub fn put_and_get_key_test<K: Eq + Debug, V: Eq + Debug, T: db::KV<K, V>>(db: T, k: K, v: V) {
    assert_eq!(db.get(&k), Ok(None));
    assert_eq!(db.put(&k, &v), Ok(()));
    assert_eq!(db.get(&k), Ok(Some(v)))
}

pub fn overwrite_key_test<K: Eq + Debug, V: Eq + Debug, T: db::KV<K, V>>(db: T, k: K, v1: V, v2: V) {
    assert_eq!(db.put(&k, &v1), Ok(()));
    assert_eq!(db.get(&k), Ok(Some(v1)));
    assert_eq!(db.put(&k, &v2), Ok(()));
    assert_eq!(db.get(&k), Ok(Some(v2)))
}

pub fn fetch_keys_by_prefix_test<K, V, DB, KGen, PKGen, VGen>(db: DB, prefix: K, non_prefixed_key:  KGen, prefixed_key: PKGen, value: VGen) -> ()
    where K: Eq + Debug + Ord, V: Eq + Debug + Ord,
          KGen: Fn() -> K, PKGen: Fn() -> K, VGen: Fn() -> V,
          DB: db::KV<K, V>
{
    let (k1, v1) = (non_prefixed_key(), value());
    let (k2, v2) = (prefixed_key(), value());
    let (k3, v3) = (non_prefixed_key(), value());
    let (k4, v4) = (prefixed_key(), value());

    assert_eq!(db.put(&k1, &v1), Ok(()));
    assert_eq!(db.put(&k2, &v2), Ok(()));
    assert_eq!(db.put(&k3, &v3), Ok(()));
    assert_eq!(db.put(&k4, &v4), Ok(()));

    assert_eq!(db.get_prefix(&prefix).sort(), vec![(k2, v2), (k4, v4)].sort())
}
