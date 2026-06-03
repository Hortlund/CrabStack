use crabstack::{
    app::AppState,
    hotkeys::{HotkeyAction, register_hotkeys},
    stack::PasteBehavior,
    system_clipboard::SystemClipboard,
    system_paste::SystemPaste,
    tray::{TrayAction, create_tray},
};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
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

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
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
                if let Err(error) = refresh_tray(&app, &mut tray) {
                    error!("failed to create tray icon: {error}");
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(UserEvent::Menu(event)) => {
                let action = tray
                    .as_ref()
                    .and_then(|tray| tray.actions_by_id.get(event.id()))
                    .copied();
                let Some(action) = action else {
                    return;
                };

                match action {
                    TrayAction::Paste => {
                        if run_paste(&app, &mut clipboard, &mut paste) {
                            update_tray(&app, &mut tray);
                        }
                    }
                    TrayAction::Clear => {
                        if let Ok(mut app) = app.lock() {
                            app.clear_stack();
                        }
                        update_tray(&app, &mut tray);
                    }
                    TrayAction::Quit => *control_flow = ControlFlow::Exit,
                }
            }
            Event::MainEventsCleared => {
                while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.state != HotKeyState::Released {
                        continue;
                    }

                    let Some(action) = hotkeys.actions_by_id.get(&event.id) else {
                        continue;
                    };

                    let changed = match action {
                        HotkeyAction::Capture => run_capture(&app, &mut clipboard),
                        HotkeyAction::Paste => run_paste(&app, &mut clipboard, &mut paste),
                    };

                    if changed {
                        update_tray(&app, &mut tray);
                    }
                }
            }
            _ => {}
        }
    });
}

fn update_tray(app: &Arc<Mutex<AppState>>, tray: &mut Option<crabstack::tray::TrayHandle>) {
    if let Err(error) = refresh_tray(app, tray) {
        error!("failed to update tray: {error}");
    }
}

fn refresh_tray(
    app: &Arc<Mutex<AppState>>,
    tray: &mut Option<crabstack::tray::TrayHandle>,
) -> Result<(), crabstack::tray::TrayError> {
    let stack_count = app.lock().map(|app| app.stack_len()).unwrap_or(0);
    let handle = create_tray(stack_count)?;
    *tray = Some(handle);
    Ok(())
}

fn run_capture(app: &Arc<Mutex<AppState>>, clipboard: &mut SystemClipboard) -> bool {
    match app.lock() {
        Ok(mut app) => match app.capture_from_clipboard(clipboard) {
            Ok(true) => {
                info!("captured clipboard text; stack size is {}", app.stack_len());
                true
            }
            Ok(false) => {
                info!("clipboard text was empty; stack unchanged");
                false
            }
            Err(error) => {
                error!("{error}");
                false
            }
        },
        Err(error) => {
            error!("app state lock failed: {error}");
            false
        }
    }
}

fn run_paste(
    app: &Arc<Mutex<AppState>>,
    clipboard: &mut SystemClipboard,
    paste: &mut SystemPaste,
) -> bool {
    match app.lock() {
        Ok(mut app) => match app.paste_stack(clipboard, paste) {
            Ok(true) => {
                info!("pasted stack; stack size is {}", app.stack_len());
                true
            }
            Ok(false) => {
                info!("paste requested with empty stack");
                false
            }
            Err(error) => {
                error!("{error}");
                false
            }
        },
        Err(error) => {
            error!("app state lock failed: {error}");
            false
        }
    }
}
