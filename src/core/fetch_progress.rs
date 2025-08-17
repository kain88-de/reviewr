//! Progress reporting for background data fetching operations

use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum FetchProgress {
    Started {
        platform_id: String,
    },
    Completed {
        platform_id: String,
        success: bool,
        items_count: Option<usize>,
        error_message: Option<String>,
    },
    AllCompleted,
}

pub type ProgressSender = mpsc::Sender<FetchProgress>;
pub type ProgressReceiver = mpsc::Receiver<FetchProgress>;

pub fn create_progress_channel() -> (ProgressSender, ProgressReceiver) {
    mpsc::channel(100) // Buffer of 100 is plenty for platform updates
}
