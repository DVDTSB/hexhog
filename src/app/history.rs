pub struct History<T> {
    pub undo_stack: Vec<Box<dyn Change<T>>>,
    pub redo_stack: Vec<Box<dyn Change<T>>>,
}

pub trait Change<T> {
    fn apply(&self, data: &mut T);
    fn revert(&self, data: &mut T);
}

impl<T> History<T> {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
    pub fn record_change(&mut self, change: Box<dyn Change<T>>) 
    where
        T: 'static + Clone,
    {
        self.undo_stack.push(change);
        self.redo_stack.clear();
    }
    pub fn undo(&mut self, data: &mut T) {
        if let Some(change) = self.undo_stack.pop() {
            change.revert(data);
            self.redo_stack.push(change);
        }
    }
    pub fn redo(&mut self, data: &mut T) {
        if let Some(change) = self.redo_stack.pop() {
            change.apply(data);
            self.undo_stack.push(change);
        }
    }
}