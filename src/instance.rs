use rusqlite::{Connection, Result, Column, NO_PARAMS};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::PooledConnection;
use serde_json::Value;
use std::collections::HashMap;

use crate::util::datatype_rust_sql;
use crate::model::Model;

pub struct Instance {
    pub json: String
}

impl Instance {
    pub fn get(conn: PooledConnection<SqliteConnectionManager>, model: &str, id: u64) -> Option<Instance> {
        let mut fields_st = conn.prepare(format!("select * from {} where id={} limit 1", model,id).as_str()).unwrap();

        let model = Model::get(&conn, model);

        let rows: Vec<Result<String>> = fields_st.query_map(NO_PARAMS, |row| {
            let mut result = String::from('{');
            let mut i = 0;
            for (k,v) in &model.fields {

                if v == "text" {
                    let val: String = row.get(i).unwrap();
                    result.push_str(format!("\"{}\": \"{}\",", k, val).as_str());

                }else if v == "integer" {
                    let val: u32 = row.get(i).unwrap();
                    result.push_str(format!("\"{}\": {},", k, val).as_str());

                }else if v == "double" {
                    let val: f64 = row.get(i).unwrap();
                    result.push_str(format!("\"{}\": {},", k, val).as_str());

                }else if v == "bool" {
                    let val: bool = row.get(i).unwrap();
                    result.push_str(format!("\"{}\": {},", k, val).as_str());
                }

                i +=1;
            }
            result.pop();
            result.push('}');

            Ok(result)

        }).unwrap().collect();

        if rows.is_empty() {
            None
        }else {
            let mut json = String::new();
            for r in rows {
                json.push_str(r.unwrap().as_str());
            }

            Some(Instance { json })
        }
    }

    pub fn delete(conn: PooledConnection<SqliteConnectionManager>, model: &str, id: u64) -> Result<(), String> {
        conn.execute(format!("delete from {} where id={}", model, id).as_str(), NO_PARAMS).unwrap();

        Ok(())
    }

    pub fn create(conn: PooledConnection<SqliteConnectionManager>, model: &str, inst: Value) -> Result<(), String> {
        let inst_obj = match inst {
            Value::Object(obj) => obj,
            _ => return Err(format!("model must be an object"))
        };

        let mut existing_fields_st = conn.prepare(format!("pragma table_info({})", model).as_str()).unwrap();
        let existing_fields: Vec<Result<(String, String)>> = existing_fields_st.query_map(NO_PARAMS, |field| {
            let name: String = field.get(1).unwrap();
            let datatype: String = field.get(2).unwrap();
            Ok((name, datatype))
        }).unwrap().collect();

        let mut exists_map = HashMap::new();
        for r in existing_fields {
            let r = r.unwrap();
            exists_map.insert(r.0, r.1);
        }

        if inst_obj.len() < exists_map.len() - 1 {
            return Err(format!("missing fields for instance of model {}", model));
        } else if inst_obj.len() > exists_map.len() - 1 {
            return Err(format!("too many fields for instance of model {}", model));
        }

        let mut cols = String::new();
        let mut vals = String::new();
        for (k, v) in inst_obj {
            match exists_map.get(k.as_str()) {
                Some(datatype) => {
                    let mut value_datatype = datatype_rust_sql(&v);

                    if value_datatype != datatype.to_owned() {
                        return Err(format!("data type miss match for field \"{}\" on instance of model \"{}\". expected \"{}\" found \"{}\"", k, model, datatype, value_datatype));
                    }
                },
                None => return Err(format!("unexpected field {} for instance of model {}", k, model))
            };

            cols.push_str(format!("{},", k).as_str());
            vals.push_str(format!("{},", v).as_str());
        }
        cols.pop();
        vals.pop();

        conn.execute(format!("insert into {} ({}) values ({})", model, cols, vals).as_str(), NO_PARAMS).unwrap();

        Ok(())
    }

    pub fn update(conn: PooledConnection<SqliteConnectionManager>, model: &str, inst: Value, query: String) -> Result<(), String> {

        Ok(())
    }
}