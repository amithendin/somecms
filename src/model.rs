use rusqlite::{Connection, Result, NO_PARAMS};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::PooledConnection;
use serde_json::Value;
use std::collections::HashMap;

pub struct Model {
    pub name: String,
    pub fields: Vec<(String, String)>
}

impl Model {
    pub fn get_all(conn: &PooledConnection<SqliteConnectionManager>) -> Vec<Model> {
        let mut models_stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let model_names = models_stmt.query_map(NO_PARAMS, |field| {
            let n: String = field.get(0).unwrap();
            Ok(n)
        }).unwrap();

        let mut model_ns = Vec::new();
        for r in model_names {
            model_ns.push(r.unwrap());
        }

        let mut models = Vec::new();
        for m in model_ns {
            let model = Model::get(conn, m.as_str());
            models.push(model);
        }

        models
    }

    pub fn get(conn: &PooledConnection<SqliteConnectionManager>, name: &str) -> Model {
        let mut existing_fields_st = conn.prepare(format!("pragma table_info({})", name).as_str()).unwrap();
        let existing_fields: Vec<Result<(String, String)>> = existing_fields_st.query_map(NO_PARAMS, |field| {
            let n: String = field.get(1).unwrap();
            let t: String = field.get(2).unwrap();
            Ok((n, t))
        }).unwrap().collect();

        let mut fields = Vec::new();
        for r in existing_fields {
            fields.push(r.unwrap());
        }

        Model { name: name.to_string(), fields }
    }

    pub fn delete(conn: PooledConnection<SqliteConnectionManager>, name: &str) -> Result<(), String> {
        conn.execute(format!("drop table {}", name).as_str(), NO_PARAMS).unwrap();

        Ok(())
    }

    pub fn update(conn: PooledConnection<SqliteConnectionManager>, model: Value, name: &str) -> Result<bool, String> {
        let model_obj = match model {
            Value::Object(obj) => obj,
            _ => return Err(format!("model must be an object"))
        };

        let mut existing_fields_st = conn.prepare(format!("pragma table_info({})", name).as_str()).unwrap();
        let existing_fields: Vec<Result<String>> = existing_fields_st.query_map(NO_PARAMS, |field| {
            let x: String = field.get(1).unwrap();
            Ok(x)
        }).unwrap().collect();

        let mut updated = false;

        if existing_fields.len() > 0 {
            updated = true;
            println!("UPDATING MODEL");

            let mut exists_map = HashMap::new();
            for r in existing_fields {
                exists_map.insert(r.unwrap(), true);
            }

            for (k, v) in model_obj {
                if !exists_map.contains_key(k.as_str()) {
                    println!("ADDING COLUMN");
                    conn.execute(format!("ALTER TABLE {} ADD COLUMN {} {};", name, k, v.as_str().unwrap()).as_str(), NO_PARAMS).unwrap();
                }
            }
        } else {
            println!("CREATING MODEL");

            let mut fields_sql = String::new();
            for (k, v) in model_obj {
                fields_sql.push_str(format!("{} {},", k, v.as_str().unwrap()).as_str());
            }
            fields_sql.pop();

            conn.execute(format!("create table if not exists {} (id integer primary key,{})", name, fields_sql).as_str(), NO_PARAMS).unwrap();
        }

        Ok(updated)
    }
}