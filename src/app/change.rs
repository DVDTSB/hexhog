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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::state::AppState;
    use crate::config::Config;

    fn make_app(data: Vec<u8>) -> App {
        App {
            config: Config::default(),
            file_name: String::new(),
            data,
            starting_line: 0,
            cursor_x: 0,
            cursor_y: 0,
            frame_height: 20,
            running: true,
            state: AppState::Move,
            buffer: [' ', ' '],
            changes: Vec::new(),
            made_changes: Vec::new(),
            is_inserting: false,
            is_selecting: false,
            selection_start: 0,
            clipboard: Vec::new(),
        }
    }

    #[test]
    fn do_change_edit_modifies_data() {
        let mut app = make_app(vec![0x00, 0x01, 0x02]);
        app.do_change(Change::Edit(1, vec![0x01], vec![0xFF]));
        assert_eq!(app.data[1], 0xFF);
        assert_eq!(app.changes.len(), 1);
    }

    #[test]
    fn undo_edit_reverts_data() {
        let mut app = make_app(vec![0x00, 0x01, 0x02]);
        app.do_change(Change::Edit(1, vec![0x01], vec![0xFF]));
        app.undo();
        assert_eq!(app.data[1], 0x01);
        assert_eq!(app.changes.len(), 0);
        assert_eq!(app.made_changes.len(), 1);
    }

    #[test]
    fn redo_reapplies_edit() {
        let mut app = make_app(vec![0x00, 0x01, 0x02]);
        app.do_change(Change::Edit(1, vec![0x01], vec![0xFF]));
        app.undo();
        app.redo();
        assert_eq!(app.data[1], 0xFF);
    }

    #[test]
    fn do_change_insert_increases_len() {
        let mut app = make_app(vec![0x00, 0x02]);
        app.do_change(Change::Insert(1, vec![0xAB]));
        assert_eq!(app.data.len(), 3);
        assert_eq!(app.data[1], 0xAB);
    }

    #[test]
    fn undo_insert_removes_bytes() {
        let mut app = make_app(vec![0x00, 0x02]);
        app.do_change(Change::Insert(1, vec![0xAB]));
        app.undo();
        assert_eq!(app.data, vec![0x00, 0x02]);
    }

    #[test]
    fn do_change_delete_decreases_len() {
        let mut app = make_app(vec![0x00, 0x01, 0x02]);
        app.do_change(Change::Delete(1, vec![0x01]));
        assert_eq!(app.data.len(), 2);
        assert_eq!(app.data[1], 0x02);
    }

    #[test]
    fn undo_delete_restores_bytes() {
        let mut app = make_app(vec![0x00, 0x01, 0x02]);
        app.do_change(Change::Delete(1, vec![0x01]));
        app.undo();
        assert_eq!(app.data, vec![0x00, 0x01, 0x02]);
    }
}
