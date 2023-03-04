use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct Data {
    value: String,
    expires_at: Option<u128>,
}
pub struct Store {
    data: HashMap<String, Data>,
}

impl Default for Store {
    fn default() -> Self {
        Store::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String, _expiry: Option<Duration>) {
        let mut expires_at: Option<u128> = None;
        if let Some(expiry) = _expiry {
            expires_at = Some(Store::get_current_timestamp() + expiry.as_millis());
        }
        self.data.insert(key, Data { value, expires_at });
    }

    pub fn get(&mut self, key: String) -> Option<&String> {
        match self.data.get(&key) {
            Some(Data {
                value,
                expires_at: None,
            }) => Some(value),
            Some(Data {
                value,
                expires_at: Some(expires_at),
            }) => {
                if *expires_at < Store::get_current_timestamp() {
                    return None;
                }
                Some(value)
            }
            None => None,
        }
    }

    fn get_current_timestamp() -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis()
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::Store;

    #[test]
    fn simple_get_on_empty_store() {
        let mut store = Store::new();
        assert_eq!(store.get(String::from("key")), None);
    }

    #[test]
    fn simple_get_set() {
        let mut store = Store::new();
        store.set(String::from("key"), String::from("value"), None);
        assert_eq!(store.get(String::from("key")), Some(&String::from("value")));
        assert_eq!(store.get(String::from("other_key")), None);
    }

    #[test]
    fn get_set_with_expiration() {
        let mut store = Store::new();
        let ten_millis = Duration::from_millis(10);
        store.set(String::from("key"), String::from("value"), Some(ten_millis));
        assert_eq!(store.get(String::from("key")), Some(&String::from("value")));

        let eleven_millis = Duration::from_millis(11);
        thread::sleep(eleven_millis);
        assert_eq!(store.get(String::from("key")), None);
    }
}
