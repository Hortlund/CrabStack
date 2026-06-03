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
cargo build
cargo run
```
