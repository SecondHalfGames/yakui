use std::ops::{Deref, DerefMut};

use crate::id::WidgetId;
use crate::widget::Widget;

/// Wraps the response returned by a widget when it is updated.
///
/// Widget responses can convey information like whether the widget was clicked,
/// is currently hovered, or had an update to its state.
pub struct Response<T: Widget> {
    inner: T::Response,

    /// The ID of the widget that responded.
    pub id: WidgetId,
}

impl<T: Widget> Response<T> {
    pub(crate) fn new(id: WidgetId, inner: T::Response) -> Self {
        Self { id, inner }
    }
}

impl<T: Widget> Deref for Response<T> {
    type Target = T::Response;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Widget> DerefMut for Response<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
