use once_cell::sync::Lazy;

pub const VERSION_MINOR: Lazy<u8> = Lazy::new(|| env!("CARGO_PKG_VERSION_MINOR").parse().unwrap());
pub const VERSION_PATCH: Lazy<u8> = Lazy::new(|| env!("CARGO_PKG_VERSION_PATCH").parse().unwrap());

pub const VERSION: Lazy<String> = Lazy::new(|| format!("{}.{}", *VERSION_MINOR, *VERSION_PATCH));