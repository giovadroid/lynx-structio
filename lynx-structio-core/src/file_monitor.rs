use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use notify::{RecursiveMode, Watcher};
use parking_lot::RwLock;
use lazy_static::lazy_static;
use crate::MonitorResult;
type UpdateCallback = dyn Fn() + Send + Sync;
type ArcFileActions = Arc<RwLock<HashMap<PathBuf, FileAction>>>;
static SHOULD_UPDATE: AtomicBool = AtomicBool::new(false);
static STOP_FLAG: AtomicBool = AtomicBool::new(false);

lazy_static! {
    pub static ref GLOBAL_FILE_MONITOR: FileMonitor = FileMonitor::new();
}

#[derive(Clone)]
struct FileAction {
    locker: Arc<AtomicBool>,
    action: Arc<UpdateCallback>,
}

///
/// Struct to monitor file changes and execute a callback when a file is modified
/// The callback is executed only once per file change
/// The callback is executed in a separate thread
///
pub struct FileMonitor {
    file_actions: ArcFileActions,
    running: Arc<AtomicBool>,
}

impl FileMonitor {

    ///
    /// Create a new FileMonitor instance
    ///
    pub fn new() -> Self {
        Self {
            file_actions: ArcFileActions::default(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    ///
    /// Register a file to be monitored
    ///
    pub fn register_file<F>(&self, path: String, monitor: F) -> MonitorResult<Arc<AtomicBool>>
        where
            F: Fn() + Send + Sync + 'static
    {
        let locker = Arc::new(AtomicBool::new(false));
        let file_path = Path::new(&path).canonicalize()?;

        self.file_actions.write().insert(file_path.clone(), FileAction {
            locker: locker.clone(),
            action: Arc::new(monitor),
        });

        if !STOP_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
            SHOULD_UPDATE.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        log::debug!("Registered file: {:?}", file_path);
        Ok(locker)
    }

    ///
    /// Start the file monitor
    ///
    pub fn listen(&self) {
        if self.running.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let listen_status = self.running.clone();
        let file_actions = self.file_actions.clone();
        tokio::task::spawn(
            async move {
                listen_status.store(true, std::sync::atomic::Ordering::Relaxed);
                let mut listen_files: Vec<PathBuf> = Vec::new();
                let file_actions_handler = file_actions.clone();
                let mut watcher = notify::recommended_watcher(move |res| {
                    match res {
                        Ok(event) => FileMonitor::handle_event(event, &file_actions_handler.read()),
                        Err(e) => log::error!("watch error: {:?}", e),
                    }
                }).expect("Failed to create watcher");

                SHOULD_UPDATE.store(true, std::sync::atomic::Ordering::Relaxed);
                log::debug!("Starting file monitor");

                while !STOP_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
                    if SHOULD_UPDATE.load(std::sync::atomic::Ordering::Relaxed) {
                        for path in listen_files.iter() {
                            if let Err(err) = watcher.unwatch(Path::new(path)){
                                log::error!("Failed to unwatch: {:?}", err);
                            }
                        }

                        let lockers = file_actions.clone().read().clone();
                        for (path, _) in lockers.iter() {
                            if let Err(err) = watcher.watch(Path::new(path), RecursiveMode::NonRecursive){
                                log::error!("Failed to watch: {:?}", err);
                                continue;
                            }
                            listen_files.push(path.clone());
                        }

                        SHOULD_UPDATE.store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                listen_status.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        );
    }

    ///
    /// Stop the file monitor
    ///
    pub fn stop(&self) {
        STOP_FLAG.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn raise_update(file_action: &FileAction) {
        if file_action.locker.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        file_action.locker.store(true, std::sync::atomic::Ordering::Relaxed);

        (file_action.action)();
        file_action.locker.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    fn handle_event(
        event: notify::Event,
        file_actions: &HashMap<PathBuf, FileAction>,
    ) {
        match event.kind {
            notify::EventKind::Access(kind) => {
                match kind {
                    notify::event::AccessKind::Close(_) => {
                        let path = event.paths.first().unwrap();
                        log::debug!("Dispatching file change by file closed: {:?}", path);

                        let file_action = file_actions.get(path).unwrap();
                        FileMonitor::raise_update(file_action);
                    }
                    _ => {}
                }
            },
            notify::EventKind::Modify(kind) => {
                match kind {
                    notify::event::ModifyKind::Data(_) => {
                        let path = event.paths.first().unwrap();
                        log::debug!("Dispatching file change by data update: {:?}", path);

                        let file_action = file_actions.get(path).unwrap();
                        FileMonitor::raise_update(file_action);
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }
}