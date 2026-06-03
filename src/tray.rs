use std::collections::HashMap;

use tray_icon::{
    BadIcon, Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuId, MenuItem, PredefinedMenuItem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Paste,
    Clear,
    Quit,
}

#[derive(Debug, thiserror::Error)]
pub enum TrayError {
    #[error(transparent)]
    Menu(#[from] tray_icon::menu::Error),
    #[error(transparent)]
    Icon(#[from] BadIcon),
    #[error(transparent)]
    Tray(#[from] tray_icon::Error),
}

pub struct TrayHandle {
    pub icon: TrayIcon,
    pub actions_by_id: HashMap<MenuId, TrayAction>,
}

pub fn create_tray(stack_count: usize) -> Result<TrayHandle, TrayError> {
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

fn default_icon() -> Result<Icon, BadIcon> {
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
