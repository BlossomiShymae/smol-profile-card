

pub fn get_timestamp() -> i64 {
    chrono::prelude::Utc::now().timestamp()
}

pub fn get_timestamp_millis() -> i64 {
    chrono::prelude::Utc::now().timestamp_millis()
}