pub use std::time::SystemTime;

lazy_static! {
    pub static ref START_TIME: SystemTime = SystemTime::now();
}

pub fn initialize_time() -> SystemTime {
    return *START_TIME;
}
