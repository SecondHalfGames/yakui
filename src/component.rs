use std::any::Any;
use std::fmt;

use glam::Vec2;
use thunderdome::Index;

use crate::layout::Constraints;

pub trait Props: Any + fmt::Debug {}

impl<T> Props for T where T: Any + fmt::Debug {}

pub trait Component: Any + fmt::Debug {
    type Props: Props;

    fn new(index: Index, props: &Self::Props) -> Self;
    fn update(&mut self, props: &Self::Props);
    fn size(&self, constraints: Constraints) -> Vec2;
}
