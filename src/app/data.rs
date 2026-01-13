use crate::app::history::{Change, History};

pub struct EditorData {
    pub data: Vec<u8>,
    pub modified: bool,
    pub cursor_position: usize,
    pub history: History<EditorChange>,
}

pub struct EditorChange {
    pub changes: Vec<Box<dyn Change<EditorData>>>,
}

impl Change<EditorData> for EditorChange {
    fn apply(&self, data: &mut EditorData) {
        for change in &self.changes {
            change.apply(data);
        }
    }

    fn revert(&self, data: &mut EditorData) {
        for change in self.changes.iter().rev() {
            change.revert(data);
        }
    }
}

pub enum BasicDataChange{
    Insert { position: usize, byte: u8 },
    Delete { position: usize, byte: u8 },
    Modify { position: usize, old_byte: u8, new_byte: u8 },
}

impl Change<EditorData> for BasicDataChange {
    fn apply(&self, data: &mut EditorData) {
        match self {
            BasicDataChange::Insert { position, byte } => {
                data.data.insert(*position, *byte);
            }
            BasicDataChange::Delete { position, .. } => {
                data.data.remove(*position);
            }
            BasicDataChange::Modify { position, new_byte, .. } => {
                data.data[*position] = *new_byte;
            }
        }
    }

    fn revert(&self, data: &mut EditorData) {
        match self {
            BasicDataChange::Insert { position, .. } => {
                data.data.remove(*position);
            }
            BasicDataChange::Delete { position, byte } => {
                data.data.insert(*position, *byte);
            }
            BasicDataChange::Modify { position, old_byte, .. } => {
                data.data[*position] = *old_byte;
            }
        }
    }
}

pub enum EditorCurosorChange {
    MoveBy(usize),
}
impl Change<EditorData> for EditorCurosorChange {
    fn apply(&self, data: &mut EditorData) {
        match self {
            EditorCurosorChange::MoveBy(offset) => {
                data.cursor_position = data.cursor_position.wrapping_add(*offset);
            }
        }
    }

    fn revert(&self, data: &mut EditorData) {
        match self {
            EditorCurosorChange::MoveBy(offset) => {
                data.cursor_position = data.cursor_position.wrapping_sub(*offset);
            }
        }
    }
}

