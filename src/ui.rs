use crate::model::Model;
use tera::{Tera, Context};
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use crate::instance::Instance;
use crate::util::DataType;
use serde_json::Value;

pub fn render_page(tera: &mut Tera, ctx: &Context, page: &str) -> Result<String, String> {
    let tmpl = format!("pub/html/{}.html",page);

    match tera.add_template_file(tmpl, Some(page)) {
        Ok(_) => {}, Err(e) => return Err(format!("template read error: {}",e))
    };

    match tera.render(page, &ctx) {
        Ok(html) => Ok(html),
        Err(e) => Err(format!("template render error: {}",e))
    }
}

pub fn types_ctx(data_types: &Vec<DataType>) -> Context {
    let mut ctx = Context::new();

    let mut data_types_arr  = Vec::new();
    for dt in data_types {
        data_types_arr.push(vec![dt.name.to_owned(), dt.meta_type.to_string()]);
    }

    ctx.insert("data_types", &data_types_arr);

    ctx
}

pub fn model_ctx(model: &Model) -> Context {
    let mut ctx = Context::new();

    ctx.insert("model", &model.name);
    ctx.insert("fields", &model.fields);

    return ctx;
}

pub fn model_html(tera: &mut Tera, model: &Model) -> Result<String, String> {
    let ctx = model_ctx(model);

    render_page(tera, &ctx, "model")
}

pub fn models_html(tera: &mut Tera, models: &Vec<Model>, data_type: &Vec<DataType>) -> Result<String, String> {
    let mut models_html = vec![];

    for model in models {
        let html = match model_html(tera, model) {
            Ok(html) => html,
            Err(e) => return Err(format!("error rendering instance for instances page: {}", e))
        };
        models_html.push(html);
    }

    let mut ctx = Context::new();
    ctx.extend(types_ctx(data_type));
    ctx.insert("models", &models_html);

    render_page(tera, &ctx, "models")
}

pub fn instance_ctx(model: &Model, inst: &Value) -> Context {
    let mut ctx = Context::new();

    let mut obj = Vec::new();
    for (k, t) in &model.fields {
        obj.push( match inst.as_object().unwrap().get(k) {
            Some(v) => {
                match v {
                    Value::String(str) => str.clone(),
                    Value::Bool(b) => format!("{}", b),
                    Value::Number(n) => format!("{}", n),
                    Value::Null => String::from("null"),
                    _=> format!("{}", v)
                }
            },
            None => "null".into()
        });
    }

    ctx.extend(model_ctx(model));
    ctx.insert("instance", &obj);

    return ctx;
}

pub fn instance_html(tera: &mut Tera, model: &Model, inst: &Value) -> Result<String, String> {
    let ctx = instance_ctx(model, inst);

    render_page(tera, &ctx, "instance")
}

pub fn instances_html(tera: &mut Tera, model: &Model, instances: &Vec<Value>) -> Result<String, String> {
    let mut insts_html = vec![];

    for inst in instances {
        let html = match instance_html(tera, model, inst) {
            Ok(html) => html,
            Err(e) => return Err(format!("error rendering instance for instances page: {}", e))
        };
        insts_html.push(html);
    }

    let mut ctx = Context::new();
    ctx.extend(model_ctx(model));
    ctx.insert("instances", &insts_html);

    render_page(tera, &ctx, "instances")
}

