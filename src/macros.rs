#[macro_export]
#[cfg(debug_assertions)]
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!("{:?}",$($t)*).into())
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! log {
    ($($t:tt)*) => {};
}

#[macro_export]
macro_rules! error{
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!("{:?}",$($t)*).into())
    };
}
