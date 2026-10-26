#[derive(Clone)]
#[non_exhaustive]
pub struct ClipboardHolder {}

impl ClipboardHolder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn copy(&self, _text: &str) {}

    pub fn paste(&self) -> Option<String> {
        None
    }

    pub fn dispose(&mut self) {}
}

impl Default for ClipboardHolder {
    fn default() -> Self {
        Self::new()
    }
}
