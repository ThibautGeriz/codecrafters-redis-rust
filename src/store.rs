use std::collections::HashMap;

pub struct Store {
    data: HashMap<String, String>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get(&mut self, key: String) -> Option<&String> {
        self.data.get(&key)
    }
}
