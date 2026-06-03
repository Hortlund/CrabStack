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
