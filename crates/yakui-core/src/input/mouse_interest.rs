use crate::event::EventInterest;
use crate::WidgetId;

#[derive(Debug)]
pub(crate) struct MouseInterest {
    layers: Vec<Vec<(WidgetId, EventInterest)>>,
    layer_stack: Vec<(WidgetId, usize)>,
}

impl MouseInterest {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            layer_stack: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.layers.clear();
        self.layer_stack.clear();
    }

    pub fn insert(&mut self, id: WidgetId, interest: EventInterest) {
        let layer = self
            .layer_stack
            .last()
            .and_then(|(_, index)| self.layers.get_mut(*index))
            .expect("cannot call MouseInterest::insert with no layers");

        layer.push((id, interest));
    }

    pub fn iter(&self) -> impl Iterator<Item = (WidgetId, EventInterest)> + '_ {
        self.layers
            .iter()
            .rev()
            .flat_map(|layer| layer.iter().copied())
    }

    pub fn current_layer_root(&self) -> Option<WidgetId> {
        self.layer_stack.last().map(|(id, _)| *id)
    }

    pub fn push_layer(&mut self, id: WidgetId) {
        let layer_index = self.layers.len();
        self.layers.push(Vec::new());
        self.layer_stack.push((id, layer_index));
    }

    pub fn pop_layer(&mut self) {
        let top = self.layer_stack.pop();
        debug_assert!(
            top.is_some(),
            "cannot call PaintLayers::pop without a corresponding push call"
        );
    }
}
