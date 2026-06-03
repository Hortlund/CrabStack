use crabstack::{
    app::AppState,
    hotkeys::{HotkeyAction, register_hotkeys},
    stack::PasteBehavior,
    system_clipboard::SystemClipboard,
    system_paste::SystemPaste,
    tray::{TrayAction, create_tray},
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

fn run_paste(app: &Arc<Mutex<AppState>>, clipboard: &mut SystemClipboard, paste: &mut SystemPaste) {
    match app.lock() {
        Ok(mut app) => match app.paste_stack(clipboard, paste) {
            Ok(true) => info!("pasted stack; stack size is {}", app.stack_len()),
            Ok(false) => info!("paste requested with empty stack"),
            Err(error) => error!("{error}"),
        },
        Err(error) => error!("app state lock failed: {error}"),
    }
}
