use rusqlite::{Connection, Result};

#[derive(Debug)]
struct KeyValuePair {
    id: i32,
    key: String,
    value: String,
}

fn main() -> Result<()> {
    let db_file_name = "database.db";
    let conn = Connection::open(db_file_name)?;

    let table_exists = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?")?.exists(["key_value_pairs"])?;

    
    if !table_exists {
        conn.execute(
            "CREATE TABLE key_value_pairs (
                id    INTEGER PRIMARY KEY,
                key   TEXT NOT NULL,
                value TEXT NOT NULL,
            )",
            (), // empty list of parameters.
        )?;
    }

    let me = KeyValuePair {
        id: 0,
        key: "asdf".to_string(),
        value: "val".to_string(),
    };
    conn.execute(
        "INSERT INTO key_value_pairs (key, value) VALUES (?1, ?2)",
        (&me.key, &me.value),
    )?;

    let mut stmt = conn.prepare("SELECT id, key, value FROM key_value_pairs")?;
    let person_iter = stmt.query_map([], |row| {
        Ok(KeyValuePair {
            id: row.get(0)?,
            key: row.get(1)?,
            value: row.get(2)?,
        })
    })?;

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
    Ok(())
}
