# Non-Blocking TUI Proposal

## Problem Statement

The current `MultiPlatformBrowser` blocks the entire TUI while fetching data from platforms (Gerrit, Jira). This creates a poor user experience where:

- Users see no progress during data loading
- TUI is completely unresponsive (can't cancel or navigate)
- Failed platforms prevent other platforms from loading
- No real-time feedback on fetch status

## Current Architecture

The existing code already implements **concurrent platform fetching** correctly using `join_all`:

```rust
impl MultiPlatformBrowser {
    pub async fn load_data(&mut self, registry: &PlatformRegistry) -> io::Result<()> {
        for platform in registry.get_configured_platforms() {
            let platform_id = platform.get_platform_id();
            match platform.get_detailed_activities(&self.employee_email, 30).await {
                Ok(activities) => {
                    self.platform_activities.insert(platform_id.to_string(), activities);
                }
                Err(e) => {
                    log::warn!("Failed to load data from {platform_id}: {e}");
                }
            }
        }
        Ok(())
    }
}
```

**The problem is NOT concurrency** - it's that the TUI blocks until all results are complete.

## Proposed Solution: Minimal Non-Blocking Approach

### Core Principle
**Keep what works, just make the TUI responsive during fetching**

### Architecture Changes

1. **Background Task**: Move concurrent fetching to a spawned task
2. **Progress Channel**: Use `tokio::sync::mpsc` for real-time updates
3. **TUI Select Loop**: Use `tokio::select!` to handle progress + user input
4. **Incremental Updates**: Update TUI as each platform completes

### Implementation Overview

```rust
// New progress update types
#[derive(Debug, Clone)]
pub enum FetchProgress {
    Started { platform_id: String },
    Completed { platform_id: String, success: bool, items_count: Option<usize> },
    AllCompleted { total_platforms: usize, successful: usize },
}

// Enhanced load_data method
impl MultiPlatformBrowser {
    pub async fn load_data_async(&mut self, registry: &PlatformRegistry) -> io::Result<()> {
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);

        // Spawn background task with existing concurrent logic
        let fetch_task = self.spawn_fetch_task(registry, progress_tx).await;

        // Non-blocking TUI loop
        self.run_fetch_loop(fetch_task, progress_rx).await
    }

    async fn spawn_fetch_task(
        &self,
        registry: &PlatformRegistry,
        progress_tx: mpsc::Sender<FetchProgress>
    ) -> JoinHandle<HashMap<String, DetailedActivities>> {
        let platforms = registry.get_configured_platforms();
        let user = self.employee_email.clone();

        tokio::spawn(async move {
            // Create concurrent tasks (existing pattern)
            let tasks: Vec<_> = platforms.iter().map(|platform| {
                let platform_id = platform.get_platform_id().to_string();
                let user = user.clone();
                let tx = progress_tx.clone();

                async move {
                    // Notify start
                    let _ = tx.send(FetchProgress::Started { platform_id: platform_id.clone() }).await;

                    // Fetch data (existing logic)
                    let result = platform.get_detailed_activities(&user, 30).await;

                    // Notify completion
                    let success = result.is_ok();
                    let items_count = result.as_ref().ok().map(|activities| {
                        activities.items_by_category.values().map(|items| items.len()).sum()
                    });

                    let _ = tx.send(FetchProgress::Completed {
                        platform_id: platform_id.clone(),
                        success,
                        items_count
                    }).await;

                    (platform_id, result)
                }
            }).collect();

            // Execute concurrently (existing pattern)
            let results = futures::future::join_all(tasks).await;

            // Collect successful results
            let mut activities = HashMap::new();
            let mut successful = 0;

            for (platform_id, result) in results {
                match result {
                    Ok(platform_activities) => {
                        activities.insert(platform_id, platform_activities);
                        successful += 1;
                    }
                    Err(_) => {} // Already logged in progress handler
                }
            }

            // Notify all complete
            let _ = progress_tx.send(FetchProgress::AllCompleted {
                total_platforms: platforms.len(),
                successful
            }).await;

            activities
        })
    }

    async fn run_fetch_loop(
        &mut self,
        mut fetch_task: JoinHandle<HashMap<String, DetailedActivities>>,
        mut progress_rx: mpsc::Receiver<FetchProgress>
    ) -> io::Result<()> {
        loop {
            tokio::select! {
                // Handle user input
                key_event = self.read_key_event() => {
                    match key_event?.code {
                        KeyCode::Esc => {
                            fetch_task.abort();
                            return Ok(());
                        }
                        KeyCode::Char('r') => {
                            // Could implement refresh individual platform
                        }
                        _ => {} // Ignore other keys during fetch
                    }
                }

                // Handle progress updates
                progress = progress_rx.recv() => {
                    match progress {
                        Some(FetchProgress::Started { platform_id }) => {
                            self.update_platform_status(&platform_id, "🔄 Fetching...");
                        }
                        Some(FetchProgress::Completed { platform_id, success, items_count }) => {
                            if success {
                                let msg = match items_count {
                                    Some(count) => format!("✅ {} items", count),
                                    None => "✅ Completed".to_string(),
                                };
                                self.update_platform_status(&platform_id, &msg);
                            } else {
                                self.update_platform_status(&platform_id, "❌ Failed");
                            }
                        }
                        Some(FetchProgress::AllCompleted { total_platforms, successful }) => {
                            self.show_completion_summary(total_platforms, successful);
                            break; // Exit progress loop
                        }
                        None => break, // Channel closed
                    }

                    // Re-render TUI with updates
                    self.render_with_progress()?;
                }

                // Check if fetch completed
                fetch_result = &mut fetch_task => {
                    match fetch_result {
                        Ok(activities) => {
                            self.platform_activities = activities;
                            break;
                        }
                        Err(e) if e.is_cancelled() => {
                            return Ok(()); // User cancelled
                        }
                        Err(e) => {
                            return Err(io::Error::other(format!("Fetch task failed: {}", e)));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
```

### UI Enhancements

```rust
impl MultiPlatformBrowser {
    fn render_with_progress(&mut self) -> io::Result<()> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Main content
                    Constraint::Length(4), // Progress/Status bar
                ])
                .split(f.area());

            // Existing header
            self.render_header(f, chunks[0]);

            // Main content (may be partial during loading)
            self.render_main_content(f, chunks[1]);

            // New: Progress bar with platform status
            self.render_progress_bar(f, chunks[2]);
        })?;

        Ok(())
    }

    fn render_progress_bar(&self, f: &mut Frame, area: Rect) {
        let progress_text = self.platform_status
            .iter()
            .map(|(platform, status)| format!("{}: {}", platform, status))
            .collect::<Vec<_>>()
            .join(" | ");

        let progress_paragraph = Paragraph::new(progress_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("📡 Platform Status"))
            .wrap(Wrap { trim: true });

        f.render_widget(progress_paragraph, area);
    }
}
```

## Benefits

### 1. **User Experience**
- ✅ Real-time progress updates for each platform
- ✅ User can cancel with ESC key
- ✅ TUI remains responsive during fetching
- ✅ Clear visual feedback on platform status

### 2. **Performance**
- ✅ Maintains existing concurrent fetching
- ✅ Failed platforms don't block successful ones
- ✅ Incremental data display as platforms complete

### 3. **Maintainability**
- ✅ Minimal changes to existing code
- ✅ Preserves current testing approach
- ✅ Clear separation of concerns

### 4. **Extensibility**
- ✅ Easy to add new platforms to registry
- ✅ Progress channel can be extended with more event types
- ✅ AI agents can wrap existing platforms without architecture changes

## Implementation Strategy

### Phase 1: Core Non-Blocking (1 week)
1. Add `FetchProgress` enum and progress channel
2. Implement `spawn_fetch_task` with existing concurrent logic
3. Implement `run_fetch_loop` with `tokio::select!`
4. Basic progress display in TUI

### Phase 2: Enhanced UX (1 week)
1. Improve progress bar with platform status
2. Add cancellation handling
3. Add retry mechanisms for failed platforms
4. Polish error messages and status display

### Phase 3: File I/O Enhancement (1 week)
1. Make error.log writing non-blocking using similar pattern
2. Add progress for file operations
3. Implement async file writing queue

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_concurrent_fetch_with_progress() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mock_platforms = create_mock_platforms();

    let task = spawn_fetch_task(mock_platforms, "testuser", tx);

    // Verify progress sequence
    assert_eq!(rx.recv().await.unwrap(), FetchProgress::Started { platform_id: "gerrit".to_string() });
    assert_eq!(rx.recv().await.unwrap(), FetchProgress::Started { platform_id: "jira".to_string() });

    // Wait for completion
    let results = task.await.unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_user_cancellation() {
    let browser = setup_test_browser();
    let registry = create_slow_mock_platforms(); // Platforms that take 5+ seconds

    let fetch_task = tokio::spawn(async move {
        browser.load_data_async(&registry).await
    });

    // Simulate ESC key after 100ms
    tokio::time::sleep(Duration::from_millis(100)).await;
    // Send KeyCode::Esc event

    // Task should complete quickly due to cancellation
    let start = Instant::now();
    let result = fetch_task.await.unwrap();
    assert!(start.elapsed() < Duration::from_millis(500));
}
```

### Integration Tests
- Test with real but fast mock platforms
- Verify concurrent execution timing
- Test cancellation scenarios
- Test error handling for failed platforms

## File Structure Changes

```
src/
├── core/
│   ├── platform.rs          # Existing platform trait
│   ├── fetch_progress.rs     # New: Progress types
│   └── async_fetch.rs        # New: Background fetch logic
├── tui/
│   ├── multi_platform_browser.rs  # Enhanced with non-blocking
│   └── progress_display.rs         # New: Progress bar components
└── main.rs                   # Unchanged
```

## Risks and Mitigations

### Risk: Increased Complexity
**Mitigation**: Changes are minimal and isolated to fetch logic. Existing patterns preserved.

### Risk: Testing Challenges
**Mitigation**: Progress channel makes testing easier - just verify message sequence.

### Risk: User Confusion
**Mitigation**: Clear progress indicators and familiar ESC-to-cancel pattern.

## Conclusion

This proposal provides a **minimal, surgical enhancement** to the existing architecture:

- **Keep**: Existing concurrent fetching, platform trait, TUI structure
- **Add**: Progress channel, non-blocking select loop, user cancellation
- **Benefit**: Responsive TUI, real-time feedback, better UX

The solution builds on proven patterns (tokio tasks + channels) without introducing complex abstractions like event buses or streams. It maintains the simplicity while delivering the responsiveness users expect from modern CLI tools.
