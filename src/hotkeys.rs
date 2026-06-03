use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
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

    for binding in &bindings {
        manager.register(binding.hotkey)?;
    }

    let actions_by_id = bindings
        .iter()
        .map(|binding| (binding.hotkey.id(), binding.action))
        .collect();

    Ok(RegisteredHotkeys {
        manager,
        actions_by_id,
        profile,
    })
}
