use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use cosmic_text::fontdb;

#[derive(Clone)]
pub struct Fonts {
    inner: Rc<RefCell<FontsInner>>,
}

struct FontsInner {
    font_system: cosmic_text::FontSystem,
}

impl Fonts {
    #[allow(unused_mut, unused_assignments)]
    fn new() -> Self {
        let mut font_system = cosmic_text::FontSystem::new_with_locale_and_db(
            sys_locale::get_locale().unwrap_or(String::from("en-US")),
            {
                let mut database = fontdb::Database::default();
                database.set_serif_family("");
                database.set_sans_serif_family("");
                database.set_cursive_family("");
                database.set_fantasy_family("");
                database.set_monospace_family("");
                database
            },
        );

        #[cfg(feature = "default-fonts")]
        {
            static DEFAULT_BYTES: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");

            font_system
                .db_mut()
                .load_font_source(fontdb::Source::Binary(Arc::from(&DEFAULT_BYTES)));
        }

        let inner = Rc::new(RefCell::new(FontsInner { font_system }));
        Self { inner }
    }

    pub fn with_system<T>(&self, f: impl FnOnce(&mut cosmic_text::FontSystem) -> T) -> T {
        let mut inner = (*self.inner).borrow_mut();

        f(&mut inner.font_system)
    }

    pub fn load_font_source(&self, source: fontdb::Source) {
        self.with_system(|font_system| font_system.db_mut().load_font_source(source));
    }
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}
