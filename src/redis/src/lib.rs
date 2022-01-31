#[macro_use]
extern crate log;

#[cfg(feature = "async")]
pub mod async_pool;
#[cfg(feature = "sync")]
pub mod sync_pool;
