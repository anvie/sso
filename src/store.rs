

use rocksdb::{DB, Writable, WriteBatch};


pub struct Store {
    path:String,
    db:DB
}

pub struct WriteBatchWrapper<'a> {
    pub wb:WriteBatch,
    db:&'a DB
}


// Storage engine, backed by RocksDB
impl Store {
    pub fn new(path:&str) -> Store {
        Store {
            path: path.to_string(),
            db: DB::open_default(path).unwrap()
        }
    }

    pub fn put(&self, key:&str, value:&str){
        self.db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    pub fn get(&self, key:&str) -> Option<String> {
        match self.db.get(key.as_bytes()){
            Ok(Some(value)) => Some(value.to_utf8().unwrap().to_string()),
            Ok(None) => None,
            _ => None
        }
    }

    pub fn del(&self, key:&str){
        self.db.delete(key.as_bytes()).unwrap();
    }

    pub fn batch(&self) -> WriteBatchWrapper {
        WriteBatchWrapper {
            wb: WriteBatch::default(),
            db: &self.db
        }
    }

    pub fn commit(&self, wbw:WriteBatchWrapper){
        self.db.write(wbw.wb);
    }
}

// wrapper for [[WriteBatch]] for easy use
// using pipe-like style, eg:
// store.batch()
//      .put(b"key1", b"value1")
//      .put(b"key2", b"value2")
//      .put(b"key3", b"value3")
//      .commit();
//
impl<'a> WriteBatchWrapper<'a> {

    pub fn put(self, key:&str, value:&str) -> Self {
        self.wb.put(key.as_bytes(), value.as_bytes());
        self
    }

    pub fn del(self, key:&str) -> Self {
        self.wb.delete(key.as_bytes()).unwrap();
        self
    }

    pub fn commit(self){
        self.db.write(self.wb);
    }
}
