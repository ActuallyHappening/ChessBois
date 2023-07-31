use once_cell::sync::Lazy;

pub static VERSION_MINOR: Lazy<u8> = Lazy::new(|| env!("CARGO_PKG_VERSION_MINOR").parse().unwrap());
pub static VERSION_PATCH: Lazy<u8> = Lazy::new(|| env!("CARGO_PKG_VERSION_PATCH").parse().unwrap());

pub static VERSION: Lazy<String> = Lazy::new(|| format!("v0.{}_{}", *VERSION_MINOR, *VERSION_PATCH));