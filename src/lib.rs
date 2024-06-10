//!
//! This crate provides a way to monitor files and execute a callback when a file is modified
//! Also, it provides a way to save and load structs to and from files
//! Requires the serde Serialize and Deserialize traits
//!
//! Also provides a proc macro to derive the Save and Load traits or to watch a file for changes
//!
//! # Example
//! ```
//! use lynx_structio::prelude::*;
//! use serde::{Serialize, Deserialize};
//! use std::path::PathBuf;
//! use std::sync::Arc;
//! use parking_lot::RwLock;
//!
//! #[derive(Serialize, Deserialize, Default, Clone)]
//! struct DataStruct {
//!    data: String,
//!    number: i32,
//! }
//!
//! // Can use FileStruct instead of FileWatch to only save and load the struct without monitoring the file
//! #[derive(FileWatch, Default, Clone)]
//! struct FileDataStruct{
//!     data: Arc<RwLock<DataStruct>>,
//! }
//!
//! impl Updatable<DataStruct> for FileDataStruct {
//!     fn update(&self, new_data: DataStruct) {
//!         self.data.write().data = new_data.data;
//!         self.data.write().number = new_data.number;
//!     }
//!
//!     fn path() -> PathBuf {
//!         PathBuf::from("data.yaml")
//!     }
//!
//!     fn content(&self) -> DataStruct {
//!         self.data.read().clone()
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let data = FileDataStruct::load().unwrap();
//!
//!     data.reload().unwrap();
//!     data.save().unwrap()
//! }
//! ```
//!

pub use lynx_structio_core::file_monitor::FileMonitor;

///
/// The monitor module provides a way to monitor files and execute a callback when a file is modified
/// The callback is executed only once per file change
/// The callback is executed in a separate thread
/// The monitor module is a wrapper around the FileMonitor struct
///
pub mod monitor {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use lynx_structio_core::file_monitor::GLOBAL_FILE_MONITOR;
    use lynx_structio_core::MonitorResult;

    ///
    /// Add a callback to be executed when a file is modified
    /// The callback is executed only once per file change
    /// The callback is executed in a separate thread
    ///
    pub fn register_file<F>(path: String, monitor: F) -> MonitorResult<Arc<AtomicBool>>
        where
            F: Fn() + Send + Sync + 'static
    {
        GLOBAL_FILE_MONITOR.register_file(path, monitor)
    }

    ///
    /// Start the file monitor
    /// This function will create a new thread to monitor the files
    /// The thread will be stopped when the stop function is called
    ///
    pub fn listen() {
        GLOBAL_FILE_MONITOR.listen();
    }

    ///
    /// Stop the file monitor
    /// This function will stop the thread that is monitoring the files
    ///
    pub fn stop() {
        GLOBAL_FILE_MONITOR.stop()
    }
}

///
/// The prelude module re-exports the core module, MonitorResult, GLOBAL_FILE_MONITOR, and the traits module
/// The prelude module is a convenience module to quickly import the most commonly used items
///
pub mod prelude {
    pub use lynx_structio_core::traits::*;
    pub use lynx_structio_macros::*;
    pub use lynx_structio_core::MonitorResult;
}

/// The derive_functions are the functions that are consumed by the derive macro
pub mod derive_functions {
    pub use lynx_structio_core::core::*;

}
