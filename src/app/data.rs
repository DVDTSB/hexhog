use crate::app::history::{Change, History};

pub struct Data {
    pub data: Vec<u8>,
    pub modified: bool,
    pub history: History<DataChange>,
}

pub enum BasicDataChange{
    Insert { position: usize, byte: u8 },
    Delete { position: usize, byte: u8 },
    Modify { position: usize, old_byte: u8, new_byte: u8 },
}

impl Change<u8> for BasicDataChange {
    fn apply(&self, data: &mut Vec<u8>) {
        match self {
            BasicDataChange::Insert { position, byte } => {
                data.insert(*position, *byte);
            }
            BasicDataChange::Delete { position, .. } => {
                data.remove(*position);
            }
            BasicDataChange::Modify { position, new_byte, .. } => {
                data[*position] = *new_byte;
            }
        }
    }

    fn revert(&self, data: &mut Vec<u8>) {
        match self {
            BasicDataChange::Insert { position, .. } => {
                data.remove(*position);
            }
            BasicDataChange::Delete { position, byte } => {
                data.insert(*position, *byte);
            }
            BasicDataChange::Modify { position, old_byte, .. } => {
                data[*position] = *old_byte;
            }
        }
    }
}

pub struct DataChange {
    pub changes: Vec<Box<dyn Change<u8>>>,
}

impl Change<u8> for DataChange {
    fn apply(&self, data: &mut Vec<u8>) {
        for change in &self.changes {
            change.apply(data);
        }
    }

    fn revert(&self, data: &mut Vec<u8>) {
        for change in self.changes.iter().rev() {
            change.revert(data);
        }
    }
}