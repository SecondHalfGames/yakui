use crate::Rect;

#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    SetViewport(Rect),
}
