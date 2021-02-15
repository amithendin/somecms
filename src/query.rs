use serde_json::Value;
use rusqlite::NO_PARAMS;
use crate::model::Model;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

fn escape_string(str: &str) -> String {
    str.replace("\"","\"\"").replace("\'","''")
}

fn parse_value_safe(val: &Value) -> String {
    match val {
        Value::String(str) =>  format!("\"{}\"",escape_string(str)),
        //Value::Object(obj) => {},
        //Value::Array(arr) => {},
        _=> escape_string(format!("{}",val).as_str())
    }
}

pub fn query_json_sql(json: Value) -> String {
    let mut q = String::new();

    let obj = json.as_object().unwrap();

    for (f,v) in obj {
        let mut op ="=";
        let mut value = String::new();

        if v.is_object() {
            let val_obj = v.as_object().unwrap();

            for (k,val) in val_obj {
                op = k.as_str();
                value = parse_value_safe(val);
            }

        }else {
            value = parse_value_safe(v)
        }

        if value.len() > 0 {
            q.push_str(format!("{}{}{},", escape_string(f), escape_string(op), value).as_str());
        }
    }
    q.pop();

    q

}

pub fn select_query(conn: &PooledConnection<SqliteConnectionManager>, model: &Model, limit: u64, offset: u64, json_body: Value) -> Result<String, String> {
    let sql_query = query_json_sql(json_body);
    let where_stmt = match sql_query.len() {
        0 => String::new(),
        _=> format!("where {}",sql_query)
    };
    
    let mut query_stmt = match conn.prepare(format!("select * from {} {} limit {} offset {}", model.name, where_stmt, limit, offset).as_str()) {
        Ok(stmt) => stmt,
        Err(e) => {
            return Err(format!("{{ \"err\": \"error preparing sql statement\", \"exception\": {:?}}}", e));
        }
    };

    let res = match query_stmt.query_map(NO_PARAMS, |row| {
        let mut result = String::from('{');
        let mut i = 0;
        for (k,v) in &model.fields {

            if v == "text" {
                let val: String = row.get(i).unwrap();
                result.push_str(format!("\"{}\": \"{}\",", k, val).as_str());

            }else if v == "int" || v == "integer" {
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

    }) {
        Ok(res) => res,
        Err(e) => {
            return Err(format!("{{ \"err\": \"error executing sql statement\", \"exception\": {:?}}}", e));
        }
    };

    let mut response = String::from('[');

    for str in res {
        response.push_str(str.unwrap().as_str());
        response.push(',');
    }
    if response.len() > 1 {
        response.pop();
    }

    response.push(']');
    Ok(response)
}