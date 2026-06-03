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
