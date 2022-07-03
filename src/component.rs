use std::any::Any;
use std::fmt;

use glam::Vec2;

use crate::layout::Constraints;

pub trait Props: Any + fmt::Debug {}

impl<T> Props for T where T: Any + fmt::Debug {}

pub trait Component<P: Props>: Any + fmt::Debug {
    fn new(props: &P) -> Self;
    fn update(&mut self, props: &P);
    fn size(&self, constraints: Constraints) -> Vec2;
}
