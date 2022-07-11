use crate::{List, ListResponse};

pub fn column<F: FnOnce()>(children: F) -> ListResponse {
    List::vertical().show(children)
}

pub fn row<F: FnOnce()>(children: F) -> ListResponse {
    List::horizontal().show(children)
}
