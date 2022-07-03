use std::any::Any;
use std::fmt;

pub trait Component<P>: Any + fmt::Debug {
    fn new(props: &P) -> Self;
    fn update(&mut self, props: &P);
}
