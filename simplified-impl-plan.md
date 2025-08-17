# Simplified Non-Blocking TUI Implementation Plan

## Guiding Principle
This plan follows the minimal approach from the `non-blocking-proposal.md`. The goal is to make the TUI responsive during data fetching with the fewest changes possible, delivering a working application at the end of Phase 1.

## Phase 1: Core Non-Blocking Functionality (Estimated: 1 Week)
**Goal:** A responsive TUI that provides real-time progress and can be canceled.

### Step 1: Create the Communication Channel
**Action:** Create a new file `src/core/fetch_progress.rs` to define the communication messages between the background task and the TUI.
```rust
// src/core/fetch_progress.rs
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum FetchProgress {
    Started { platform_id: String },
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
    mpsc::channel(100) // Buffer of 100 is plenty
}
```
**TDD Focus:**
- Write unit tests to verify the `FetchProgress` enum.
- Write a test to ensure the channel can be created and a message can be sent/received.

### Step 2: Implement the Background Fetch Task
**Action:** In `src/tui/multi_platform_browser.rs`, create a new method `spawn_fetch_task` that wraps the existing concurrent fetching logic in a `tokio::spawn` block. This task will send `FetchProgress` messages.

```rust
// In src/tui/multi_platform_browser.rs
async fn spawn_fetch_task(
    &self,
    registry: PlatformRegistry,
    progress_tx: ProgressSender,
) -> JoinHandle<HashMap<String, DetailedActivities>> {
    let platforms = registry.get_configured_platforms();
    let user = self.employee_email.clone();

    tokio::spawn(async move {
        let tasks: Vec<_> = platforms.iter().map(|platform| {
            let platform_id = platform.get_platform_id().to_string();
            let user = user.clone();
            let tx = progress_tx.clone();

            async move {
                // 1. Send Started message
                let _ = tx.send(FetchProgress::Started { platform_id: platform_id.clone() }).await;

                // 2. Fetch data (existing logic)
                let result = platform.get_detailed_activities(&user, 30).await;

                // 3. Send Completed message
                let _ = tx.send(FetchProgress::Completed {
                    platform_id: platform_id.clone(),
                    success: result.is_ok(),
                    items_count: result.as_ref().ok().map(|a| a.items_by_category.values().map(Vec::len).sum()),
                    error_message: result.as_ref().err().map(|e| e.to_string()),
                }).await;

                (platform_id, result)
            }
        }).collect();

        // Execute concurrently and collect results
        let results = futures::future::join_all(tasks).await;
        let mut activities = HashMap::new();
        for (platform_id, result) in results {
            if let Ok(platform_activities) = result {
                activities.insert(platform_id, platform_activities);
            }
        }

        // 4. Send AllCompleted message
        let _ = progress_tx.send(FetchProgress::AllCompleted).await;
        activities
    })
}
```
**TDD Focus:**
- Write an integration test that calls this function with mock platforms.
- Assert that the correct sequence of `FetchProgress` messages is received on a mock channel.
- Assert that the final `HashMap` of activities is correct.

### Step 3: Create the Non-Blocking TUI Loop
**Action:** Create the main `load_data_async` method and a `run_fetch_loop` that uses `tokio::select!` to handle both user input and progress messages simultaneously.

```rust
// In src/tui/multi_platform_browser.rs
pub async fn load_data_async(&mut self, registry: &PlatformRegistry) -> io::Result<()> {
    let (progress_tx, mut progress_rx) = create_progress_channel();
    let fetch_task = self.spawn_fetch_task(registry.clone(), progress_tx).await;
    self.run_fetch_loop(fetch_task, &mut progress_rx).await
}

async fn run_fetch_loop(
    &mut self,
    mut fetch_task: JoinHandle<HashMap<String, DetailedActivities>>,
    progress_rx: &mut ProgressReceiver,
) -> io::Result<()> {
    loop {
        // Re-render the TUI on every loop iteration to show updates
        self.render_progress_ui()?;

        tokio::select! {
            // 1. Handle user input (e.g., cancellation)
            key_event = read_key_event_async() => {
                if let Ok(Some(KeyEvent { code: KeyCode::Esc, .. })) = key_event {
                    fetch_task.abort();
                    break;
                }
            }

            // 2. Handle progress updates from the fetch task
            progress = progress_rx.recv() => {
                match progress {
                    Some(FetchProgress::Started { platform_id }) => {
                        self.platform_status.insert(platform_id, "⏳ Fetching...".to_string());
                    }
                    Some(FetchProgress::Completed { platform_id, success, items_count, .. }) => {
                        let status = if success {
                            format!("✅ Done ({} items)", items_count.unwrap_or(0))
                        } else {
                            "❌ Failed".to_string()
                        };
                        self.platform_status.insert(platform_id, status);
                    }
                    Some(FetchProgress::AllCompleted) | None => {
                        // Fetch is done, break the loop
                        break;
                    }
                }
            }
        }
    }

    // Wait for the task to finish and get the data
    if let Ok(activities) = fetch_task.await {
        self.platform_activities = activities;
    }

    Ok(())
}
```
**TDD Focus:**
- Write a UI test (can be a higher-level integration test) that simulates a slow fetch.
- Send a `KeyCode::Esc` event during the test.
- Assert that the `run_fetch_loop` exits quickly without waiting for the slow fetch to complete.

---
**End of Phase 1 Outcome:** A working application that meets the primary goal. The TUI is responsive, shows live progress, and the fetch operation can be canceled.

---

## Phase 2: UX and Error Handling Polish (Estimated: 3-4 Days)
**Goal:** Refine the UI and make error handling more robust and user-friendly.

### Step 1: Improve Progress Display
**Action:** Create a dedicated `render_progress_bar` function to display the status in a more structured and visually appealing way, rather than just printing text. This could be a simple list or a `Gauge` widget from `ratatui`.

**TDD Focus:**
- This is primarily a visual change, but unit tests can be written for the logic that calculates the progress percentage for a `Gauge`.

### Step 2: Enhance Error Reporting
**Action:** When a `FetchProgress::Completed` message arrives with `success: false`, use the `error_message` field to display a more informative error in the UI.

**TDD Focus:**
- Modify the `spawn_fetch_task` test to include a mock platform that returns an error.
- Assert that the `FetchProgress::Completed` message contains the correct error string.

### Step 3: Final Polish
**Action:** Clean up the UI, ensure smooth transitions between the loading state and the final data display, and add comments where necessary.

---
**End of Phase 2 Outcome:** A polished, user-friendly feature that is ready for release.
