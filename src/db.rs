use std::path::{Path, PathBuf};

use crate::state::State;

// pub trait AutoRandrDB {
//     fn get_db_connection(cache_dir: &PathBuf) -> anyhow::Result<sqlite::Connection>;
// }

// impl AutoRandrDB for sqlite::Connection {
//     fn get_db_connection(cache_dir: &PathBuf) -> anyhow::Result<sqlite::Connection> {
//         Ok(())
//     }
// }
pub struct AutoRandrDB {
    db_connection: rusqlite::Connection,
}

impl AutoRandrDB {
    pub fn new(cache_dir: &PathBuf) -> anyhow::Result<Self> {
        let db_path = cache_dir.as_path().join(Path::new("modes.sqlite3"));
        let db_connection = rusqlite::Connection::open(db_path)?;
        db_connection.execute(
            "CREATE TABLE IF NOT EXISTS modes (
                id INTEGER PRIMARY KEY,
                outputs TEXT NOT NULL,
                state TEXT NOT NULL
            )",
            (),
        )?;
        Ok(Self { db_connection })
    }

    // pub fn get_state(&self, output_sign: String) -> State {

    // }

    pub fn get_state(&self, output_sign: &str) -> anyhow::Result<State> {
        let state = self.db_connection.query_row(
            "SELECT state FROM modes WHERE outputs = ?",
            (output_sign,),
            |row| {
                let state_json: String = row.get(0)?;
                Ok(state_json)
            },
        );
        let state: State = serde_json::from_str(&state?)?;
        Ok(state)
    }

    pub fn save_state(&self, state: &State) -> anyhow::Result<()> {
        let previous_state = self.get_state(&state.outputs_sign());
        if let Ok(previous_state) = previous_state {
            if previous_state == *state {
                return Ok(());
            } else {
                self.db_connection.execute(
                    "UPDATE modes SET state = ? WHERE outputs = ?",
                    (&serde_json::to_string(state)?, &state.outputs_sign()),
                )?;
                return Ok(());
            }
        }
        let state_json = serde_json::to_string(state)?;
        self.db_connection.execute(
            "INSERT INTO modes (outputs, state) VALUES (?, ?)",
            (&state.outputs_sign(), &state_json),
        )?;
        Ok(())
    }

    pub fn remove_state(&self, state: &State) -> anyhow::Result<()> {
        self.db_connection.execute(
            "DELETE FROM modes WHERE outputs = ?",
            (state.outputs_sign(),),
        )?;
        Ok(())
    }
}
