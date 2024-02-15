use std::ops::{Deref, DerefMut};

use crate::id::WidgetId;

/// Wraps the response returned by a widget when it is updated.
///
/// Widget responses can convey information like whether the widget was clicked,
/// is currently hovered, or had an update to its state.
#[derive(Debug)]
pub struct Response<T> {
    inner: T,

    /// The ID of the widget that responded.
    pub id: WidgetId,
}

impl<T> Response<T> {
    pub(crate) fn new(id: WidgetId, inner: T) -> Self {
        Self { id, inner }
    }

    /// Unwrap the response into the underlying type.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Request that the widget with this response should receive focus.
    pub fn request_focus(&self) {
        crate::context::dom().request_focus(self.id);
    }
}

impl<T> Deref for Response<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Response<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
