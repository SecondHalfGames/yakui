use std::any::Any;
use std::fmt;

pub trait Props: Any + fmt::Debug {}

impl<T> Props for T where T: Any + fmt::Debug {}

pub trait Component<P: Props>: Any + fmt::Debug {
    fn new(props: &P) -> Self;
    fn update(&mut self, props: &P);
}
