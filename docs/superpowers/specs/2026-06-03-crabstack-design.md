# CrabStack Design

## Goal

CrabStack is a small cross-platform Rust tray app for stacking copied text snippets into a newline-separated paste buffer. It is intentionally not a full clipboard manager.

The core workflow is:

1. Copy one line of text in any application.
2. Press the capture hotkey to append the current clipboard text to the stack.
3. Repeat for more lines.
4. Press the paste hotkey to paste the stacked text into the focused application.
5. Clear the stack automatically after a successful paste by default.

## Platform Scope

CrabStack targets:

- macOS
- Linux
- Windows

The first version should build and run as a native Rust desktop utility on all three platforms. CI should compile and package release artifacts for each target OS.

## Architecture

CrabStack will be a pure Rust tray/menu-bar application.

The app will have these units:

- `AppState`: owns the current stack and behavior settings.
- `ClipboardService`: reads and writes plain text clipboard contents.
- `HotkeyService`: registers global capture and paste hotkeys.
- `PasteService`: triggers paste into the currently focused application after writing stacked text to the clipboard.
- `TrayService`: exposes basic tray/menu-bar actions and current stack count.
- `ConfigService`: deferred until settings are added after v1.

These units should communicate through small interfaces so platform-specific details stay isolated.

## Default Hotkeys

The default hotkeys are:

- Capture current clipboard text: `Cmd+Shift+C` on macOS, `Ctrl+Shift+C` on Linux and Windows.
- Paste stack: `Cmd+Shift+V` on macOS, `Ctrl+Shift+V` on Linux and Windows.

If the primary defaults cannot be registered on a platform, fall back to `Cmd+Option+C` and `Cmd+Option+V` on macOS, or `Ctrl+Alt+C` and `Ctrl+Alt+V` on Linux and Windows.

## Capture Behavior

When the capture hotkey fires:

- Read the current clipboard as text.
- Ignore empty or whitespace-only clipboard contents.
- Trim trailing line endings from the captured text.
- Append the result as one stack item.
- Preserve internal newlines if the clipboard contains multiple lines.

No automatic clipboard monitoring is included in v1. This avoids accidentally collecting unrelated clipboard data.

## Paste Behavior

When the paste hotkey fires:

- If the stack is empty, do nothing.
- Join stack items with `\n`.
- Write the joined text to the clipboard.
- Trigger the platform paste shortcut into the currently focused application.
- Clear the stack after paste by default.

The initial behavior mode is `paste_and_clear`. The core stack logic should represent paste behavior as a small enum so `paste_and_keep` can be added without rewriting paste flow later.

## Tray/Menu Behavior

CrabStack runs primarily as a background tray/menu-bar app.

The tray menu should include:

- Current stack count.
- Paste stack.
- Clear stack.
- Quit.

No normal main window is required for v1.

## Settings

Settings are intentionally minimal.

V1 will not include a settings UI or persisted settings. It will use hardcoded defaults for hotkeys and paste behavior.

When settings are added later, they should cover only:

- Capture hotkey.
- Paste hotkey.
- Paste behavior: clear after paste or keep after paste.

Future settings should be stored in a simple config file in the platform-appropriate user config directory.

## Error Handling

Clipboard, hotkey, tray, and paste simulation operations can fail differently by OS.

Expected handling:

- Hotkey registration failure should be visible through logs and, if feasible, a tray notification or disabled menu state.
- Clipboard read failure should skip capture and log the error.
- Clipboard write or paste trigger failure should leave the stack intact.
- Empty stack paste should be a no-op.

## CI And Release Builds

GitHub Actions should build on:

- `macos-latest`
- `ubuntu-latest`
- `windows-latest`

CI should include:

- Formatting check.
- Linting with Clippy.
- Unit tests.
- Release builds for all three platforms.
- Uploadable artifacts for successful release builds.

Packaging can start simple. A runnable binary per OS is enough for v1 unless packaging tools are straightforward to add.

## Testing

The first implementation should include unit tests for:

- Capturing empty clipboard text.
- Capturing normal single-line text.
- Capturing text with trailing newlines.
- Joining multiple stack items.
- Clearing after paste.
- Keeping the stack intact when paste fails.

Platform integration behavior should be verified manually at first because global hotkeys and paste simulation are OS-sensitive.

## Non-Goals

The first version will not include:

- Clipboard history.
- Automatic clipboard monitoring.
- Rich text or image clipboard support.
- Search.
- Cloud sync.
- Multi-profile workflows.
- A large settings UI.
- A full application window.
