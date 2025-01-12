//! IMAP extensions.

pub mod compress;
pub mod enable;
pub mod idle;
pub mod r#move;
pub mod quota;
#[cfg(feature = "ext_sort_thread")]
pub mod sort;
#[cfg(feature = "ext_sort_thread")]
pub mod thread;
pub mod unselect;
