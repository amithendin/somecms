use serde_json::Value;

pub fn datatype_rust_sql(v: &Value) -> String {
    let mut value_datatype = "text";

    if v.is_string() {
        value_datatype = "text";
    }else if v.is_boolean() {
        value_datatype = "bool";
    }else if v.is_i64() || v.is_u64() {
        value_datatype = "int";
    }else if v.is_f64() {
        value_datatype = "double";
    }

    return value_datatype.to_string();
}
