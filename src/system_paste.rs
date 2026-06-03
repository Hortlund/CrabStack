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
        let modifier = paste_modifier();

        self.enigo
            .key(modifier, Press)
            .map_err(|error| AppError::PasteTrigger(error.to_string()))?;

        let click_result = self
            .enigo
            .key(Key::Unicode('v'), Click)
            .map_err(|error| error.to_string());
        let release_result = self
            .enigo
            .key(modifier, Release)
            .map_err(|error| error.to_string());

        match (click_result, release_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(click_error), Ok(())) => Err(AppError::PasteTrigger(click_error)),
            (Ok(()), Err(release_error)) => Err(AppError::PasteTrigger(release_error)),
            (Err(click_error), Err(release_error)) => Err(AppError::PasteTrigger(format!(
                "{click_error}; modifier release failed: {release_error}"
            ))),
        }
    }
}

fn paste_modifier() -> Key {
    #[cfg(target_os = "macos")]
    {
        Key::Meta
    }

    #[cfg(not(target_os = "macos"))]
    {
        Key::Control
    }
}
