use std::ops::{Deref, DerefMut};

use thunderdome::Index;

use crate::widget::Widget;

pub struct Response<T: Widget> {
    inner: T::Response,
    pub index: Index,
}

impl<T: Widget> Response<T> {
    pub(crate) fn new(index: Index, inner: T::Response) -> Self {
        Self { index, inner }
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
