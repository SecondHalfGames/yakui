use std::cell::RefCell;
use std::rc::Rc;

use arboard::Clipboard;

#[derive(Clone)]
pub struct ClipboardHolder {
    inner: Option<Rc<RefCell<Clipboard>>>,
}

impl ClipboardHolder {
    pub fn new() -> Self {
        let inner = match Clipboard::new() {
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
        callback: impl FnOnce(&mut Clipboard) -> Result<T, arboard::Error>,
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

impl Default for ClipboardHolder {
    fn default() -> Self {
        Self::new()
    }
}
