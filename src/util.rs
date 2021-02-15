use serde_json::Value;
use std::fmt;

pub enum MetaType {
    Primitive,
    Emum,
    Model
}

impl fmt::Display for MetaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetaType::Primitive => write!(f, "primitive"),
            MetaType::Emum => write!(f, "enum"),
            MetaType::Model => write!(f, "model")
        }
    }
}

pub struct DataType {
    pub name: String,
    pub meta_type: MetaType
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.meta_type)
    }
}

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
