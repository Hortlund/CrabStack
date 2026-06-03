# CrabStack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a minimal Rust tray app that stacks copied text snippets and pastes the joined stack with global hotkeys on macOS, Linux, and Windows.

**Architecture:** Keep the stack and paste orchestration as testable pure Rust modules, then wrap OS-sensitive clipboard, paste, hotkey, and tray behavior behind small services. The binary owns the event loop and wires tray/menu events plus global hotkey events into the same app state.

**Tech Stack:** Rust, Cargo, `arboard`, `global-hotkey`, `tray-icon`, `tao`, `enigo`, `thiserror`, `tracing`, GitHub Actions.

---

## File Structure

- `Cargo.toml`: crate metadata and dependencies.
- `.gitignore`: Rust build output and local editor files.
- `README.md`: short usage, hotkeys, Linux dependency note, and CI/build commands.
- `src/lib.rs`: public module declarations for testable app code.
- `src/stack.rs`: pure stack state, capture trimming, joining, clear/keep paste behavior.
- `src/app.rs`: app orchestration around stack, clipboard, and paste service traits.
- `src/system_clipboard.rs`: `arboard` implementation of the clipboard trait.
- `src/system_paste.rs`: `enigo` implementation of the paste trigger trait.
- `src/hotkeys.rs`: global hotkey registration and hotkey event mapping.
- `src/tray.rs`: tray icon, tray menu construction, and menu action mapping.
- `src/main.rs`: event loop, shared app state, service wiring, and command dispatch.
- `.github/workflows/ci.yml`: formatting, linting, tests, and release builds for macOS, Linux, and Windows.

## Task 1: Scaffold The Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `README.md`

- [ ] **Step 1: Initialize the Cargo binary crate**

Run:

```bash
cargo init --bin --name crabstack .
```

Expected: Cargo creates `Cargo.toml` and `src/main.rs`.

- [ ] **Step 2: Add runtime dependencies**

Run:

```bash
cargo add arboard global-hotkey tray-icon tao enigo thiserror tracing tracing-subscriber
```

Expected: Cargo updates `Cargo.toml` and creates `Cargo.lock`.

- [ ] **Step 3: Add `.gitignore`**

Write `./.gitignore`:

```gitignore
/target/
.DS_Store
*.swp
*.swo
```

- [ ] **Step 4: Add library module shell**

Write `./src/lib.rs`:

```rust
pub mod app;
pub mod hotkeys;
pub mod stack;
pub mod system_clipboard;
pub mod system_paste;
pub mod tray;
```

- [ ] **Step 5: Keep the binary temporarily minimal**

Write `./src/main.rs`:

```rust
fn main() {
    println!("CrabStack scaffold ready");
}
```

- [ ] **Step 6: Add a short README**

Write `./README.md`:

````markdown
# CrabStack

CrabStack is a tiny Rust tray app for stacking copied text snippets into one newline-separated paste.

Default workflow:

1. Copy text in any app.
2. Press the capture hotkey to add clipboard text to the stack.
3. Press the paste hotkey to paste the stack and clear it.

Default hotkeys:

- macOS: `Cmd+Shift+C` capture, `Cmd+Shift+V` paste
- Linux/Windows: `Ctrl+Shift+C` capture, `Ctrl+Shift+V` paste
- Fallback: `Cmd+Option+C/V` on macOS, `Ctrl+Alt+C/V` on Linux/Windows

Linux builds need GTK/AppIndicator/XDO development packages. On Ubuntu:

```bash
sudo apt install libgtk-3-dev libxdo-dev libayatana-appindicator3-dev
```

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run
```
````

- [ ] **Step 7: Verify scaffold builds**

Run:

```bash
cargo check
```

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock .gitignore README.md src/main.rs src/lib.rs
git commit -m "chore: scaffold Rust app"
```

Expected: one commit containing the crate scaffold.

## Task 2: Implement Pure Stack Logic With Tests

**Files:**
- Create: `src/stack.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing stack tests**

Write `./src/stack.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteBehavior {
    ClearAfterPaste,
    KeepAfterPaste,
}

#[derive(Debug, Clone)]
pub struct TextStack {
    items: Vec<String>,
    paste_behavior: PasteBehavior,
}

impl TextStack {
    pub fn new(paste_behavior: PasteBehavior) -> Self {
        Self {
            items: Vec::new(),
            paste_behavior,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PasteBehavior, TextStack};

    #[test]
    fn ignores_empty_clipboard_text() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);

        assert!(!stack.capture(""));
        assert!(!stack.capture("   \n\t"));

        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn captures_normal_single_line_text() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);

        assert!(stack.capture("Line1"));

        assert_eq!(stack.len(), 1);
        assert_eq!(stack.joined(), Some("Line1".to_string()));
    }

    #[test]
    fn trims_trailing_line_endings_only() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);

        assert!(stack.capture("  Line1  \r\n\n"));

        assert_eq!(stack.joined(), Some("  Line1  ".to_string()));
    }

    #[test]
    fn joins_multiple_stack_items_with_newlines() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);

        stack.capture("Line1");
        stack.capture("Line2");
        stack.capture("Line3");

        assert_eq!(stack.joined(), Some("Line1\nLine2\nLine3".to_string()));
    }

    #[test]
    fn clears_after_successful_paste_when_configured() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);
        stack.capture("Line1");

        stack.mark_paste_succeeded();

        assert!(stack.is_empty());
    }

    #[test]
    fn keeps_after_successful_paste_when_configured() {
        let mut stack = TextStack::new(PasteBehavior::KeepAfterPaste);
        stack.capture("Line1");

        stack.mark_paste_succeeded();

        assert_eq!(stack.joined(), Some("Line1".to_string()));
    }

    #[test]
    fn clear_removes_all_items() {
        let mut stack = TextStack::new(PasteBehavior::KeepAfterPaste);
        stack.capture("Line1");
        stack.capture("Line2");

        stack.clear();

        assert!(stack.is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test stack
```

Expected: FAIL with missing methods such as `capture`, `len`, `is_empty`, `joined`, `mark_paste_succeeded`, and `clear`.

- [ ] **Step 3: Implement stack methods**

Replace `./src/stack.rs` with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteBehavior {
    ClearAfterPaste,
    KeepAfterPaste,
}

#[derive(Debug, Clone)]
pub struct TextStack {
    items: Vec<String>,
    paste_behavior: PasteBehavior,
}

impl TextStack {
    pub fn new(paste_behavior: PasteBehavior) -> Self {
        Self {
            items: Vec::new(),
            paste_behavior,
        }
    }

    pub fn capture(&mut self, clipboard_text: &str) -> bool {
        if clipboard_text.trim().is_empty() {
            return false;
        }

        let trimmed = clipboard_text.trim_end_matches(['\r', '\n']).to_string();
        self.items.push(trimmed);
        true
    }

    pub fn joined(&self) -> Option<String> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.join("\n"))
        }
    }

    pub fn mark_paste_succeeded(&mut self) {
        if self.paste_behavior == PasteBehavior::ClearAfterPaste {
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{PasteBehavior, TextStack};

    #[test]
    fn ignores_empty_clipboard_text() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);

        assert!(!stack.capture(""));
        assert!(!stack.capture("   \n\t"));

        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn captures_normal_single_line_text() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);

        assert!(stack.capture("Line1"));

        assert_eq!(stack.len(), 1);
        assert_eq!(stack.joined(), Some("Line1".to_string()));
    }

    #[test]
    fn trims_trailing_line_endings_only() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);

        assert!(stack.capture("  Line1  \r\n\n"));

        assert_eq!(stack.joined(), Some("  Line1  ".to_string()));
    }

    #[test]
    fn joins_multiple_stack_items_with_newlines() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);

        stack.capture("Line1");
        stack.capture("Line2");
        stack.capture("Line3");

        assert_eq!(stack.joined(), Some("Line1\nLine2\nLine3".to_string()));
    }

    #[test]
    fn clears_after_successful_paste_when_configured() {
        let mut stack = TextStack::new(PasteBehavior::ClearAfterPaste);
        stack.capture("Line1");

        stack.mark_paste_succeeded();

        assert!(stack.is_empty());
    }

    #[test]
    fn keeps_after_successful_paste_when_configured() {
        let mut stack = TextStack::new(PasteBehavior::KeepAfterPaste);
        stack.capture("Line1");

        stack.mark_paste_succeeded();

        assert_eq!(stack.joined(), Some("Line1".to_string()));
    }

    #[test]
    fn clear_removes_all_items() {
        let mut stack = TextStack::new(PasteBehavior::KeepAfterPaste);
        stack.capture("Line1");
        stack.capture("Line2");

        stack.clear();

        assert!(stack.is_empty());
    }
}
```

- [ ] **Step 4: Verify stack tests pass**

Run:

```bash
cargo test stack
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add src/stack.rs src/lib.rs
git commit -m "feat: add text stack"
```

Expected: one commit containing the tested stack logic.

## Task 3: Implement App Orchestration With Mocked Services

**Files:**
- Create: `src/app.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing app orchestration tests**

Write `./src/app.rs`:

```rust
use crate::stack::{PasteBehavior, TextStack};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppError {
    #[error("clipboard read failed: {0}")]
    ClipboardRead(String),
    #[error("clipboard write failed: {0}")]
    ClipboardWrite(String),
    #[error("paste trigger failed: {0}")]
    PasteTrigger(String),
}

pub trait ClipboardService {
    fn read_text(&mut self) -> Result<String, AppError>;
    fn write_text(&mut self, text: &str) -> Result<(), AppError>;
}

pub trait PasteService {
    fn trigger_paste(&mut self) -> Result<(), AppError>;
}

pub struct AppState {
    stack: TextStack,
}

impl AppState {
    pub fn new(paste_behavior: PasteBehavior) -> Self {
        Self {
            stack: TextStack::new(paste_behavior),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppError, AppState, ClipboardService, PasteService};
    use crate::stack::PasteBehavior;

    #[derive(Default)]
    struct FakeClipboard {
        text: String,
        written: Vec<String>,
        read_error: Option<AppError>,
        write_error: Option<AppError>,
    }

    impl ClipboardService for FakeClipboard {
        fn read_text(&mut self) -> Result<String, AppError> {
            if let Some(error) = self.read_error.take() {
                Err(error)
            } else {
                Ok(self.text.clone())
            }
        }

        fn write_text(&mut self, text: &str) -> Result<(), AppError> {
            if let Some(error) = self.write_error.take() {
                Err(error)
            } else {
                self.written.push(text.to_string());
                Ok(())
            }
        }
    }

    #[derive(Default)]
    struct FakePaste {
        calls: usize,
        error: Option<AppError>,
    }

    impl PasteService for FakePaste {
        fn trigger_paste(&mut self) -> Result<(), AppError> {
            self.calls += 1;
            if let Some(error) = self.error.take() {
                Err(error)
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn capture_reads_clipboard_and_adds_text() {
        let mut app = AppState::new(PasteBehavior::ClearAfterPaste);
        let mut clipboard = FakeClipboard {
            text: "Line1".to_string(),
            ..FakeClipboard::default()
        };

        assert!(app.capture_from_clipboard(&mut clipboard).unwrap());

        assert_eq!(app.stack_len(), 1);
    }

    #[test]
    fn capture_skips_when_clipboard_read_fails() {
        let mut app = AppState::new(PasteBehavior::ClearAfterPaste);
        let mut clipboard = FakeClipboard {
            read_error: Some(AppError::ClipboardRead("not available".to_string())),
            ..FakeClipboard::default()
        };

        let error = app.capture_from_clipboard(&mut clipboard).unwrap_err();

        assert_eq!(error, AppError::ClipboardRead("not available".to_string()));
        assert_eq!(app.stack_len(), 0);
    }

    #[test]
    fn paste_empty_stack_is_noop() {
        let mut app = AppState::new(PasteBehavior::ClearAfterPaste);
        let mut clipboard = FakeClipboard::default();
        let mut paste = FakePaste::default();

        assert!(!app.paste_stack(&mut clipboard, &mut paste).unwrap());

        assert!(clipboard.written.is_empty());
        assert_eq!(paste.calls, 0);
    }

    #[test]
    fn paste_writes_joined_stack_triggers_paste_and_clears() {
        let mut app = AppState::new(PasteBehavior::ClearAfterPaste);
        let mut clipboard = FakeClipboard::default();
        let mut paste = FakePaste::default();

        app.stack_mut().capture("Line1");
        app.stack_mut().capture("Line2");

        assert!(app.paste_stack(&mut clipboard, &mut paste).unwrap());

        assert_eq!(clipboard.written, vec!["Line1\nLine2"]);
        assert_eq!(paste.calls, 1);
        assert_eq!(app.stack_len(), 0);
    }

    #[test]
    fn paste_write_failure_leaves_stack_intact() {
        let mut app = AppState::new(PasteBehavior::ClearAfterPaste);
        let mut clipboard = FakeClipboard {
            write_error: Some(AppError::ClipboardWrite("denied".to_string())),
            ..FakeClipboard::default()
        };
        let mut paste = FakePaste::default();

        app.stack_mut().capture("Line1");

        let error = app.paste_stack(&mut clipboard, &mut paste).unwrap_err();

        assert_eq!(error, AppError::ClipboardWrite("denied".to_string()));
        assert_eq!(paste.calls, 0);
        assert_eq!(app.stack_len(), 1);
    }

    #[test]
    fn paste_trigger_failure_leaves_stack_intact() {
        let mut app = AppState::new(PasteBehavior::ClearAfterPaste);
        let mut clipboard = FakeClipboard::default();
        let mut paste = FakePaste {
            error: Some(AppError::PasteTrigger("automation denied".to_string())),
            ..FakePaste::default()
        };

        app.stack_mut().capture("Line1");

        let error = app.paste_stack(&mut clipboard, &mut paste).unwrap_err();

        assert_eq!(error, AppError::PasteTrigger("automation denied".to_string()));
        assert_eq!(clipboard.written, vec!["Line1"]);
        assert_eq!(app.stack_len(), 1);
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test app
```

Expected: FAIL with missing `capture_from_clipboard`, `paste_stack`, `stack_len`, and `stack_mut`.

- [ ] **Step 3: Implement app orchestration**

Replace the `impl AppState` block in `./src/app.rs` with:

```rust
impl AppState {
    pub fn new(paste_behavior: PasteBehavior) -> Self {
        Self {
            stack: TextStack::new(paste_behavior),
        }
    }

    pub fn capture_from_clipboard(
        &mut self,
        clipboard: &mut impl ClipboardService,
    ) -> Result<bool, AppError> {
        let text = clipboard.read_text()?;
        Ok(self.stack.capture(&text))
    }

    pub fn paste_stack(
        &mut self,
        clipboard: &mut impl ClipboardService,
        paste: &mut impl PasteService,
    ) -> Result<bool, AppError> {
        let Some(joined) = self.stack.joined() else {
            return Ok(false);
        };

        clipboard.write_text(&joined)?;
        paste.trigger_paste()?;
        self.stack.mark_paste_succeeded();
        Ok(true)
    }

    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }

    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    pub fn stack_mut(&mut self) -> &mut TextStack {
        &mut self.stack
    }
}
```

Keep the existing trait definitions and tests from Step 1.

- [ ] **Step 4: Verify app tests pass**

Run:

```bash
cargo test app
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add src/app.rs src/lib.rs
git commit -m "feat: add app orchestration"
```

Expected: one commit containing app orchestration and mocked-service tests.

## Task 4: Add System Clipboard And Paste Services

**Files:**
- Create: `src/system_clipboard.rs`
- Create: `src/system_paste.rs`

- [ ] **Step 1: Implement the system clipboard wrapper**

Write `./src/system_clipboard.rs`:

```rust
use crate::app::{AppError, ClipboardService};

pub struct SystemClipboard {
    clipboard: arboard::Clipboard,
}

impl SystemClipboard {
    pub fn new() -> Result<Self, AppError> {
        let clipboard = arboard::Clipboard::new()
            .map_err(|error| AppError::ClipboardRead(error.to_string()))?;
        Ok(Self { clipboard })
    }
}

impl ClipboardService for SystemClipboard {
    fn read_text(&mut self) -> Result<String, AppError> {
        self.clipboard
            .get_text()
            .map_err(|error| AppError::ClipboardRead(error.to_string()))
    }

    fn write_text(&mut self, text: &str) -> Result<(), AppError> {
        self.clipboard
            .set_text(text.to_string())
            .map_err(|error| AppError::ClipboardWrite(error.to_string()))
    }
}
```

- [ ] **Step 2: Implement the paste trigger wrapper**

Write `./src/system_paste.rs`:

```rust
use crate::app::{AppError, PasteService};
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

pub struct SystemPaste {
    enigo: Enigo,
}

impl SystemPaste {
    pub fn new() -> Result<Self, AppError> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|error| AppError::PasteTrigger(error.to_string()))?;
        Ok(Self { enigo })
    }
}

impl PasteService for SystemPaste {
    fn trigger_paste(&mut self) -> Result<(), AppError> {
        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;

        #[cfg(not(target_os = "macos"))]
        let modifier = Key::Control;

        self.enigo
            .key(modifier, Press)
            .map_err(|error| AppError::PasteTrigger(error.to_string()))?;
        self.enigo
            .key(Key::Unicode('v'), Click)
            .map_err(|error| AppError::PasteTrigger(error.to_string()))?;
        self.enigo
            .key(modifier, Release)
            .map_err(|error| AppError::PasteTrigger(error.to_string()))?;

        Ok(())
    }
}
```

- [ ] **Step 3: Verify the service modules compile**

Run:

```bash
cargo check
```

Expected: PASS.

- [ ] **Step 4: Run all unit tests**

Run:

```bash
cargo test
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add src/system_clipboard.rs src/system_paste.rs
git commit -m "feat: add system clipboard and paste services"
```

Expected: one commit containing service wrappers.

## Task 5: Add Hotkey Mapping

**Files:**
- Create: `src/hotkeys.rs`

- [ ] **Step 1: Write hotkey action mapping tests**

Write `./src/hotkeys.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    Capture,
    Paste,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyProfile {
    Primary,
    Fallback,
}

#[cfg(test)]
mod tests {
    use super::{HotkeyAction, HotkeyProfile};

    #[test]
    fn primary_profile_contains_capture_and_paste() {
        let bindings = super::bindings_for_profile(HotkeyProfile::Primary);

        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].action, HotkeyAction::Capture);
        assert_eq!(bindings[1].action, HotkeyAction::Paste);
    }

    #[test]
    fn fallback_profile_contains_capture_and_paste() {
        let bindings = super::bindings_for_profile(HotkeyProfile::Fallback);

        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].action, HotkeyAction::Capture);
        assert_eq!(bindings[1].action, HotkeyAction::Paste);
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test hotkeys
```

Expected: FAIL with missing `bindings_for_profile`.

- [ ] **Step 3: Implement hotkey registration model**

Replace `./src/hotkeys.rs` with:

```rust
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    Capture,
    Paste,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyProfile {
    Primary,
    Fallback,
}

#[derive(Debug, Clone)]
pub struct HotkeyBinding {
    pub action: HotkeyAction,
    pub hotkey: HotKey,
}

pub struct RegisteredHotkeys {
    pub manager: GlobalHotKeyManager,
    pub actions_by_id: HashMap<u32, HotkeyAction>,
    pub profile: HotkeyProfile,
}

pub fn bindings_for_profile(profile: HotkeyProfile) -> Vec<HotkeyBinding> {
    #[cfg(target_os = "macos")]
    let primary_modifier = Modifiers::SUPER | Modifiers::SHIFT;
    #[cfg(not(target_os = "macos"))]
    let primary_modifier = Modifiers::CONTROL | Modifiers::SHIFT;

    #[cfg(target_os = "macos")]
    let fallback_modifier = Modifiers::SUPER | Modifiers::ALT;
    #[cfg(not(target_os = "macos"))]
    let fallback_modifier = Modifiers::CONTROL | Modifiers::ALT;

    let modifier = match profile {
        HotkeyProfile::Primary => primary_modifier,
        HotkeyProfile::Fallback => fallback_modifier,
    };

    vec![
        HotkeyBinding {
            action: HotkeyAction::Capture,
            hotkey: HotKey::new(Some(modifier), Code::KeyC),
        },
        HotkeyBinding {
            action: HotkeyAction::Paste,
            hotkey: HotKey::new(Some(modifier), Code::KeyV),
        },
    ]
}

pub fn register_hotkeys() -> Result<RegisteredHotkeys, global_hotkey::Error> {
    try_register_profile(HotkeyProfile::Primary)
        .or_else(|_| try_register_profile(HotkeyProfile::Fallback))
}

fn try_register_profile(profile: HotkeyProfile) -> Result<RegisteredHotkeys, global_hotkey::Error> {
    let manager = GlobalHotKeyManager::new()?;
    let bindings = bindings_for_profile(profile);
    let mut actions_by_id = HashMap::new();

    for binding in bindings {
        manager.register(binding.hotkey)?;
        actions_by_id.insert(binding.hotkey.id(), binding.action);
    }

    Ok(RegisteredHotkeys {
        manager,
        actions_by_id,
        profile,
    })
}

#[cfg(test)]
mod tests {
    use super::{HotkeyAction, HotkeyProfile};

    #[test]
    fn primary_profile_contains_capture_and_paste() {
        let bindings = super::bindings_for_profile(HotkeyProfile::Primary);

        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].action, HotkeyAction::Capture);
        assert_eq!(bindings[1].action, HotkeyAction::Paste);
    }

    #[test]
    fn fallback_profile_contains_capture_and_paste() {
        let bindings = super::bindings_for_profile(HotkeyProfile::Fallback);

        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].action, HotkeyAction::Capture);
        assert_eq!(bindings[1].action, HotkeyAction::Paste);
    }
}
```

- [ ] **Step 4: Verify hotkey tests and compilation**

Run:

```bash
cargo test hotkeys
cargo check
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add src/hotkeys.rs
git commit -m "feat: add hotkey registration"
```

Expected: one commit containing hotkey mapping and fallback registration.

## Task 6: Add Tray Menu Model And Tray Builder

**Files:**
- Create: `src/tray.rs`

- [ ] **Step 1: Write tray action tests**

Write `./src/tray.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Paste,
    Clear,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::TrayAction;

    #[test]
    fn tray_actions_are_stable() {
        assert_eq!(TrayAction::Paste, TrayAction::Paste);
        assert_ne!(TrayAction::Paste, TrayAction::Clear);
        assert_ne!(TrayAction::Clear, TrayAction::Quit);
    }
}
```

- [ ] **Step 2: Run tests to verify the tray module is wired**

Run:

```bash
cargo test tray
```

Expected: PASS.

- [ ] **Step 3: Implement tray menu construction**

Replace `./src/tray.rs` with:

```rust
use std::collections::HashMap;
use tray_icon::{
    menu::{Menu, MenuId, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Paste,
    Clear,
    Quit,
}

pub struct TrayHandle {
    pub icon: TrayIcon,
    pub actions_by_id: HashMap<MenuId, TrayAction>,
}

pub fn create_tray(stack_count: usize) -> Result<TrayHandle, tray_icon::Error> {
    let menu = Menu::new();
    let count_item = MenuItem::new(format!("Stack: {stack_count}"), false, None);
    let paste_item = MenuItem::new("Paste stack", true, None);
    let clear_item = MenuItem::new("Clear stack", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    menu.append(&count_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&paste_item)?;
    menu.append(&clear_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let mut actions_by_id = HashMap::new();
    actions_by_id.insert(paste_item.id().clone(), TrayAction::Paste);
    actions_by_id.insert(clear_item.id().clone(), TrayAction::Clear);
    actions_by_id.insert(quit_item.id().clone(), TrayAction::Quit);

    let icon = TrayIconBuilder::new()
        .with_tooltip("CrabStack")
        .with_menu(Box::new(menu))
        .with_icon(default_icon()?)
        .build()?;

    Ok(TrayHandle {
        icon,
        actions_by_id,
    })
}

fn default_icon() -> Result<Icon, tray_icon::BadIcon> {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let offset = (y * width + x) * 4;
            let on = (8..24).contains(&x) && (6..26).contains(&y);
            rgba[offset] = if on { 220 } else { 0 };
            rgba[offset + 1] = if on { 90 } else { 0 };
            rgba[offset + 2] = if on { 70 } else { 0 };
            rgba[offset + 3] = if on { 255 } else { 0 };
        }
    }

    Icon::from_rgba(rgba, width as u32, height as u32)
}

#[cfg(test)]
mod tests {
    use super::TrayAction;

    #[test]
    fn tray_actions_are_stable() {
        assert_eq!(TrayAction::Paste, TrayAction::Paste);
        assert_ne!(TrayAction::Paste, TrayAction::Clear);
        assert_ne!(TrayAction::Clear, TrayAction::Quit);
    }
}
```

- [ ] **Step 4: Verify tray module compiles**

Run:

```bash
cargo test tray
cargo check
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add src/tray.rs
git commit -m "feat: add tray menu"
```

Expected: one commit containing tray action mapping and tray construction.

## Task 7: Wire The Runtime Event Loop

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace the scaffold binary with runtime wiring**

Write `./src/main.rs`:

```rust
use crabstack::{
    app::AppState,
    hotkeys::{register_hotkeys, HotkeyAction},
    stack::PasteBehavior,
    system_clipboard::SystemClipboard,
    system_paste::SystemPaste,
    tray::{create_tray, TrayAction},
};
use global_hotkey::GlobalHotKeyEvent;
use std::sync::{Arc, Mutex};
use tao::{
    event::{Event, StartCause},
    event_loop::{ControlFlow, EventLoopBuilder},
};
use tracing::{error, info};
use tray_icon::menu::MenuEvent;

enum UserEvent {
    Menu(MenuEvent),
}

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .build()
        .expect("failed to create event loop");
    let proxy = event_loop.create_proxy();

    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::Menu(event));
    }));

    let app = Arc::new(Mutex::new(AppState::new(PasteBehavior::ClearAfterPaste)));
    let mut clipboard = match SystemClipboard::new() {
        Ok(clipboard) => clipboard,
        Err(error) => {
            error!("{error}");
            return;
        }
    };
    let mut paste = match SystemPaste::new() {
        Ok(paste) => paste,
        Err(error) => {
            error!("{error}");
            return;
        }
    };
    let hotkeys = match register_hotkeys() {
        Ok(hotkeys) => {
            info!("registered {:?} hotkeys", hotkeys.profile);
            hotkeys
        }
        Err(error) => {
            error!("failed to register hotkeys: {error}");
            return;
        }
    };

    let mut tray = None;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                let stack_count = app.lock().map(|app| app.stack_len()).unwrap_or(0);
                match create_tray(stack_count) {
                    Ok(handle) => tray = Some(handle),
                    Err(error) => {
                        error!("failed to create tray icon: {error}");
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
            Event::UserEvent(UserEvent::Menu(event)) => {
                let Some(tray) = tray.as_ref() else {
                    return;
                };
                let Some(action) = tray.actions_by_id.get(event.id()) else {
                    return;
                };

                match action {
                    TrayAction::Paste => run_paste(&app, &mut clipboard, &mut paste),
                    TrayAction::Clear => {
                        if let Ok(mut app) = app.lock() {
                            app.clear_stack();
                        }
                    }
                    TrayAction::Quit => *control_flow = ControlFlow::Exit,
                }
            }
            Event::MainEventsCleared => {
                while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    let Some(action) = hotkeys.actions_by_id.get(&event.id) else {
                        continue;
                    };

                    match action {
                        HotkeyAction::Capture => run_capture(&app, &mut clipboard),
                        HotkeyAction::Paste => run_paste(&app, &mut clipboard, &mut paste),
                    }
                }
            }
            _ => {}
        }
    });
}

fn run_capture(app: &Arc<Mutex<AppState>>, clipboard: &mut SystemClipboard) {
    match app.lock() {
        Ok(mut app) => match app.capture_from_clipboard(clipboard) {
            Ok(true) => info!("captured clipboard text; stack size is {}", app.stack_len()),
            Ok(false) => info!("clipboard text was empty; stack unchanged"),
            Err(error) => error!("{error}"),
        },
        Err(error) => error!("app state lock failed: {error}"),
    }
}

fn run_paste(
    app: &Arc<Mutex<AppState>>,
    clipboard: &mut SystemClipboard,
    paste: &mut SystemPaste,
) {
    match app.lock() {
        Ok(mut app) => match app.paste_stack(clipboard, paste) {
            Ok(true) => info!("pasted stack; stack size is {}", app.stack_len()),
            Ok(false) => info!("paste requested with empty stack"),
            Err(error) => error!("{error}"),
        },
        Err(error) => error!("app state lock failed: {error}"),
    }
}
```

- [ ] **Step 2: Verify runtime compiles**

Run:

```bash
cargo check
```

Expected: PASS.

- [ ] **Step 3: Run all tests**

Run:

```bash
cargo test
```

Expected: PASS.

- [ ] **Step 4: Commit**

Run:

```bash
git add src/main.rs
git commit -m "feat: wire tray runtime"
```

Expected: one commit containing runtime event-loop wiring.

## Task 8: Add CI For Three Desktop Platforms

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Add GitHub Actions workflow**

Write `./.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: ["main"]
  pull_request:
    branches: ["main"]
  workflow_dispatch:

jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [macos-latest, ubuntu-latest, windows-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Install Linux system dependencies
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libxdo-dev libayatana-appindicator3-dev

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Tests
        run: cargo test --all-targets --all-features

      - name: Release build
        run: cargo build --release

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: crabstack-${{ runner.os }}
          path: |
            target/release/crabstack
            target/release/crabstack.exe
          if-no-files-found: error
```

- [ ] **Step 2: Run local formatting, linting, and tests**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Expected: PASS locally.

- [ ] **Step 3: Commit**

Run:

```bash
git add .github/workflows/ci.yml
git commit -m "ci: build CrabStack on desktop platforms"
```

Expected: one commit containing CI workflow.

## Task 9: Manual Smoke Test And Release Notes

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Run the app locally**

Run:

```bash
cargo run
```

Expected: app starts and a CrabStack tray/menu-bar icon appears.

- [ ] **Step 2: Smoke test capture and paste**

Manual steps:

```text
1. Open a text editor.
2. Copy "Line1".
3. Press the capture hotkey.
4. Copy "Line2".
5. Press the capture hotkey.
6. Focus the text editor.
7. Press the paste hotkey.
8. Confirm the editor receives:
   Line1
   Line2
9. Press the paste hotkey again.
10. Confirm nothing is pasted because the stack cleared.
```

Expected: pasted text matches exactly and the second paste is a no-op.

- [ ] **Step 3: Document manual verification**

Append to `./README.md`:

````markdown
## Manual Smoke Test

Before tagging a release, run:

```bash
cargo run
```

Then verify:

- The tray/menu-bar icon appears.
- Capture hotkey adds the current clipboard text.
- Paste hotkey pastes stacked text as newline-separated lines.
- Paste clears the stack by default.
- Tray menu can paste, clear, and quit.
````

- [ ] **Step 4: Commit**

Run:

```bash
git add README.md
git commit -m "docs: add smoke test"
```

Expected: one commit containing release verification notes.

## Final Verification

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
git status --short
```

Expected:

- Format check passes.
- Clippy passes without warnings.
- Tests pass.
- Release build succeeds.
- `git status --short` shows no uncommitted changes.
