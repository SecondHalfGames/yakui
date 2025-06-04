use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct ClipboardHolder {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    inner: Option<Rc<RefCell<arboard::Clipboard>>>,
}


#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
impl ClipboardHolder {
    pub fn new() -> Self {
        let inner = match arboard::Clipboard::new() {
            Ok(c) => Some(Rc::new(RefCell::new(c))),
            Err(err) => {
                log::error!("Failed to open clipboard: {err:?}");
                None
            }
        };

        Self { inner }
    }

    pub fn copy(&self, text: &str) {
        self.operate(|clipboard| {
            clipboard.set_text(text)?;
            Ok(())
        });
    }

    pub fn paste(&self) -> Option<String> {
        self.operate(|clipboard| clipboard.get_text())
    }

    pub fn dispose(&mut self) {
        // Clipboard must be explicitly dropped when the program terminates, so
        // we take it here to drop it.
        let _ = self.inner.take();
    }

    fn operate<T>(
        &self,
        callback: impl FnOnce(&mut arboard::Clipboard) -> Result<T, arboard::Error>,
    ) -> Option<T> {
        let inner = self.inner.as_ref().map(|inner| inner.borrow_mut());

        match inner {
            Some(mut clipboard) => match callback(&mut clipboard) {
                Ok(v) => Some(v),
                Err(err) => {
                    log::error!("Failed to operate on clipboard: {err:?}");
                    None
                }
            },
            None => {
                log::error!("Failed to operate on clipboard: clipboard failed to open");
                None
            }
        }
    }
}

// Stubbed out implementation awaiting:
// https://github.com/1Password/arboard/pull/103
// https://github.com/1Password/arboard/pull/171
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
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
