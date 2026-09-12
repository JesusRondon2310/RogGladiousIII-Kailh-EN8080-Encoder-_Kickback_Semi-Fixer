use std::env;
use std::sync::LazyLock;

pub static DEBUG: LazyLock<bool> = LazyLock::new(|| env::var_os("KICKBACK_DEBUG").is_some());

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if *$crate::helpers::trace::DEBUG { print!($($arg)*); }
    };
}
