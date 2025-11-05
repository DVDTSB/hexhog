use crate::app::App;

#[derive(Debug, Clone)]
pub enum Change {
    Edit(usize, Vec<u8>, Vec<u8>),
    Insert(usize, Vec<u8>),
    Delete(usize, Vec<u8>),
}

impl App {
    pub fn do_change(&mut self, change: Change) {
        self.changes.push(change.clone());
        match change {
            Change::Edit(idx, _old, new) => self.replace_data(idx, new),
            Change::Insert(idx, new) => self.insert_data(idx, new),
            Change::Delete(idx, old) => self.delete_data(idx, old.len()),
        }
    }

    pub fn undo_change(&mut self, change: Change) {
        self.made_changes.push(change.clone());
        match change {
            Change::Edit(idx, old, _new) => self.replace_data(idx, old),
            Change::Insert(idx, new) => self.delete_data(idx, new.len()),
            Change::Delete(idx, old) => self.insert_data(idx, old),
        }
    }

    pub fn undo(&mut self) {
        if let Some(change) = self.changes.pop() {
            self.undo_change(change);
        }
    }

    pub fn redo(&mut self) {
        if let Some(change) = self.made_changes.pop() {
            self.do_change(change);
        }
    }
}
