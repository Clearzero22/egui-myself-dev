//! Async clipboard storage wrapper.
//!
//! This module provides a wrapper that executes storage operations on a background thread,
//! keeping the UI thread responsive.

use super::store::{Store, Result as StoreResult, Error as StoreError};
use super::item::ClipboardItem;
use std::sync::mpsc;
use std::thread;

/// Commands that can be sent to the background storage thread.
#[derive(Debug)]
enum StoreCommand {
    /// Get all items
    GetAll(mpsc::Sender<Vec<ClipboardItem>>),
    /// Get a page of items with pagination
    GetPage { offset: usize, limit: usize, response: mpsc::Sender<Vec<ClipboardItem>> },
    /// Add an item
    Add(ClipboardItem, mpsc::Sender<StoreResult<()>>),
    /// Remove an item
    Remove(usize, mpsc::Sender<StoreResult<()>>),
    /// Update an item
    Update(usize, ClipboardItem, mpsc::Sender<StoreResult<()>>),
    /// Clear all items
    Clear(mpsc::Sender<StoreResult<()>>),
    /// Get item count
    Count(mpsc::Sender<usize>),
    /// Shutdown the thread
    Shutdown,
}

/// Async wrapper for a Store that runs operations on a background thread.
///
/// This keeps the UI thread responsive by offloading storage I/O to a background thread.
pub struct AsyncStore<S: Store + Send + 'static> {
    /// Sender for commands to the background thread
    sender: mpsc::Sender<StoreCommand>,
    /// Whether the background thread is still running
    _running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Phantom data to keep the type parameter
    _phantom: std::marker::PhantomData<S>,
}

impl<S: Store + Send + 'static> AsyncStore<S> {
    /// Create a new async store wrapper.
    ///
    /// This spawns a background thread that handles all storage operations.
    ///
    /// # Arguments
    ///
    /// * `store` - The underlying store to wrap (e.g., SqliteStore)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use clipboard_history::core::async_store::AsyncStore;
    /// use clipboard_history::core::SqliteStore;
    ///
    /// let store = SqliteStore::new().unwrap();
    /// let async_store = AsyncStore::new(store);
    /// ```
    pub fn new(mut store: S) -> Self {
        let (sender, receiver) = mpsc::channel();
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

        let running_clone = running.clone();
        thread::spawn(move || {
            // Process commands on the background thread
            while running_clone.load(std::sync::atomic::Ordering::Relaxed) {
                match receiver.recv() {
                    Ok(cmd) => {
                        // Handle command
                        match cmd {
                            StoreCommand::GetAll(response) => {
                                let items = store.get_all();
                                let _ = response.send(items);
                            }
                            StoreCommand::GetPage { offset, limit, response } => {
                                let items = store.get_page(offset, limit);
                                let _ = response.send(items);
                            }
                            StoreCommand::Add(item, response) => {
                                let result = store.add(item);
                                let _ = response.send(result);
                            }
                            StoreCommand::Remove(index, response) => {
                                let result = store.remove(index);
                                let _ = response.send(result);
                            }
                            StoreCommand::Update(index, item, response) => {
                                let result = store.update(index, item);
                                let _ = response.send(result);
                            }
                            StoreCommand::Clear(response) => {
                                let result = store.clear();
                                let _ = response.send(result);
                            }
                            StoreCommand::Count(response) => {
                                let count = store.len();
                                let _ = response.send(count);
                            }
                            StoreCommand::Shutdown => {
                                running_clone.store(false, std::sync::atomic::Ordering::Relaxed);
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // Channel closed, exit the thread
                        break;
                    }
                }
            }
        });

        Self {
            sender,
            _running: running,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get all items asynchronously.
    ///
    /// This sends a request to the background thread and waits for the response.
    pub fn get_all(&self) -> Vec<ClipboardItem> {
        let (sender, receiver) = mpsc::channel();
        let _ = self.sender.send(StoreCommand::GetAll(sender));
        receiver.recv().unwrap_or_default()
    }

    /// Get a page of items asynchronously.
    pub fn get_page(&self, offset: usize, limit: usize) -> Vec<ClipboardItem> {
        let (sender, receiver) = mpsc::channel();
        let _ = self.sender.send(StoreCommand::GetPage { offset, limit, response: sender });
        receiver.recv().unwrap_or_default()
    }

    /// Add an item asynchronously.
    pub fn add(&mut self, item: ClipboardItem) -> StoreResult<()> {
        let (sender, receiver) = mpsc::channel();
        let _ = self.sender.send(StoreCommand::Add(item, sender));
        receiver.recv().unwrap_or(Err(StoreError::Other("Channel closed".to_string())))
    }

    /// Remove an item asynchronously.
    pub fn remove(&mut self, index: usize) -> StoreResult<()> {
        let (sender, receiver) = mpsc::channel();
        let _ = self.sender.send(StoreCommand::Remove(index, sender));
        receiver.recv().unwrap_or(Err(StoreError::Other("Channel closed".to_string())))
    }

    /// Update an item asynchronously.
    pub fn update(&mut self, index: usize, item: ClipboardItem) -> StoreResult<()> {
        let (sender, receiver) = mpsc::channel();
        let _ = self.sender.send(StoreCommand::Update(index, item, sender));
        receiver.recv().unwrap_or(Err(StoreError::Other("Channel closed".to_string())))
    }

    /// Clear all items asynchronously.
    pub fn clear(&mut self) -> StoreResult<()> {
        let (sender, receiver) = mpsc::channel();
        let _ = self.sender.send(StoreCommand::Clear(sender));
        receiver.recv().unwrap_or(Err(StoreError::Other("Channel closed".to_string())))
    }

    /// Get the number of items asynchronously.
    pub fn len(&self) -> usize {
        let (sender, receiver) = mpsc::channel();
        let _ = self.sender.send(StoreCommand::Count(sender));
        receiver.recv().unwrap_or(0)
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Shutdown the background thread.
    pub fn shutdown(self) {
        let _ = self.sender.send(StoreCommand::Shutdown);
    }
}

impl<S: Store + Send + 'static> Drop for AsyncStore<S> {
    fn drop(&mut self) {
        let _ = self.sender.send(StoreCommand::Shutdown);
    }
}
