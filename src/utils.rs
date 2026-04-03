#[macro_export]
macro_rules! unwrap_option {
    ($t:expr) => {
        match $t {
            Some(v) => v,
            None => return,
        }
    };
}
#[macro_export]
macro_rules! unwrap_result {
    ($t:expr) => {
        match $t {
            Some(v) => v,
            None => return Ok(()),
        }
    };
}
