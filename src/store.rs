

use rocksdb::{DB, Writable};


pub struct Store {
    path:String,
    db:DB
}

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

}
