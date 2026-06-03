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
}
