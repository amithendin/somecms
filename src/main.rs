#[macro_use]
extern crate tokio;
extern crate hyper;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate url;
#[macro_use]
extern crate rusqlite;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate tera;
mod model;
mod instance;
mod util;
mod somelogger;
mod query;
mod ui;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Error, Response, Server, Method, StatusCode, header};
use hyper::service::{make_service_fn, service_fn};
use rusqlite::{Connection, Result, NO_PARAMS};
use std::thread;
use r2d2_sqlite::SqliteConnectionManager;
use serde_json::Value;
use std::collections::HashMap;
use crate::model::Model;
use crate::instance::Instance;
use rusqlite::ffi::ErrorCode::CannotOpen;
use r2d2::PooledConnection;
use crate::somelogger::SomeLogger;
use log::{SetLoggerError, LevelFilter};
use log::{info, warn, error};
use tokio_util::codec::BytesCodec;
use tera::{Tera, Context, Function};

async fn service(conn: PooledConnection<SqliteConnectionManager>, _req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = String::new();
    let mut content_type = "application/json";
    let path_str = _req.uri().path().to_string();
    let path: Vec<&str> = path_str.split("/").filter(|str| str.len() > 0).collect();

    let params: HashMap<String, String> = _req
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);

    info!("{} {}", _req.method(), path_str);

    match _req.method() {
        &Method::GET => {

            if path_str == "/favicon.ico" {
                if let Ok(file) = tokio::fs::File::open("pub/favicon.ico").await {
                    let stream = tokio_util::codec::FramedRead::new(file, BytesCodec::new());
                    let body = Body::wrap_stream(stream);

                    return Ok(Response::new(body));
                }
            }

            if path.len() > 0 {
                if path[0] == "api" {
                    if path.len() == 2 {
                        //model
                        let model = Model::get(&conn, path[1]);
                        response = String::from('{');
                        for r in model.fields {
                            response.push_str(format!("\"{}\": \"{}\",", r.0, r.1).as_str());
                        }
                        response.pop();
                        response.push('}');

                    }else if path.len() == 3 {
                        //instance
                        let id = match path[2].parse::<u64>() {
                            Ok(v) => v,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error parsing id\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        let instance = Instance::get(conn, path[1], id);
                        response = match instance {
                            Some(inst) => inst.json,
                            None => "null".to_string()
                        };
                    }

                }else if path[0] == "ui" {
                    let mut tera = Tera::default();
                    let container_tmpl = "pub/html/container.html";
                    match tera.add_template_file(container_tmpl, Some("container")) {
                        Ok(_) => {}, Err(e) => return Ok(Response::new(format!("template read error: {}",e).into()))
                    };
                    let mut body = String::new();
                    content_type = "text/html";

                    if path.len() == 2 {
                        let model = Model::get(&conn, path[1]);

                        let instances: Value = match query::select_query(&conn, &model, 10, 0, json!({})) {
                            Ok(json) => {
                                match serde_json::from_str(&json) {
                                    Ok(obj) => obj,
                                    Err(e) => return Ok(Response::new(format!("error parsing json from query results: {}", e).into()))
                                }
                            },
                            Err(e) => return Ok(Response::new(format!("query error: {}",e).into()))
                        };

                        body = match ui::instances_html(&mut tera, &model, instances.as_array().unwrap()) {
                            Ok(html) => html,
                            Err(e) => return Ok(Response::new(e.into()))
                        };

                    }else {
                        let models = Model::get_all(&conn);
                        let data_types = Model::get_types(&conn);

                        body = match ui::models_html(&mut tera, &models, &data_types) {
                            Ok(html) => html,
                            Err(e) => return Ok(Response::new(e.into()))
                        };
                    }

                    let mut tmp_ctx = Context::new();
                    tmp_ctx.insert("content", &body);
                    response = tera.render("container", &tmp_ctx).unwrap();

                }else if path[0] == "pub" {
                    let file_path = format!("pub/{}", &path[1..].join("/"));
                    if file_path.ends_with(".css") {
                        content_type = "text/css";
                    }else if file_path.ends_with(".js") {
                        content_type = "application/javascript";
                    }

                    if let Ok(file) = tokio::fs::File::open(file_path).await {
                        let stream = tokio_util::codec::FramedRead::new(file, BytesCodec::new());
                        let body = Body::wrap_stream(stream);

                        return Ok(Response::new(body));

                    }else {
                        return Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body("Not Found".into())
                            .unwrap());
                    }

                }else if path[0] == "types" {
                    let data_types = Model::get_types(&conn);

                    response = String::from('[');
                    for dt in data_types {
                        let dt_json = format!("{{ \"name\": \"{}\", \"meta_type\": \"{}\" }},", dt.name, dt.meta_type);
                        response.push_str(dt_json.as_str());
                    }
                    if response.len() > 1 {
                        response.pop();
                    }
                    response.push(']');
                }
            }
        },
        &Method::DELETE => {
            if path.len() > 1 {
                if path[0] == "api" {
                    if path.len() == 2 {
                        //model
                        response = match Model::delete(conn, path[1]) {
                            Ok(_) => "{\"msg\": \"deleted\"}".to_string(),
                            Err(e) => e
                        };

                    }else if path.len() == 3 {
                        //instance
                        let id = match path[2].parse::<u64>() {
                            Ok(v) => v,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error parsing id\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        response = match Instance::delete(conn, path[1], id) {
                            Ok(_) => "{\"msg\": \"deleted\"}".to_string(),
                            Err(e) => e
                        };
                    }
                }
            }
        },
        &Method::PUT => {
            if path.len() > 1 {
                if path[0] == "api" {
                    if path.len() == 2 {
                        //create instance
                        let b = match hyper::body::to_bytes(_req).await {
                            Ok(b) => b,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error receiving request\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        let json_body: Value = match serde_json::from_slice(b.as_ref()) {
                            Ok(v) => v,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error parsing json\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        response = match Instance::create(conn, path[1], json_body) {
                            Ok(_) => format!("{{\"msg\": \"created\"}}"),
                            Err(e) => format!("{{ \"err\": \"data structure error\", \"exception\": {:?}}}", e)
                        };

                    }else if path.len() == 3 {
                        //update instance
                        let b = match hyper::body::to_bytes(_req).await {
                            Ok(b) => b,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error receiving request\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        let json_body: Value = match serde_json::from_slice(b.as_ref()) {
                            Ok(v) => v,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error parsing json\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        response = match Instance::update(conn, path[1], json_body, format!("id=\"{}\"", path[2])) {
                            Ok(_) => format!("{{\"msg\": \"created\"}}"),
                            Err(e) => format!("{{ \"err\": \"data structure error\", \"exception\": {:?}}}", e)
                        };
                    }
                }
            }
        },
        &Method::POST => {
            if path.len() > 1 {
                if path[0] == "api" {
                    if path.len() == 2 {
                        //update model
                        let b = match hyper::body::to_bytes(_req).await {
                            Ok(b) => b,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error receiving request\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        let json_body: Value = match serde_json::from_slice(b.as_ref()) {
                            Ok(v) => v,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error parsing json\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        response = match Model::update(conn, json_body, path[1]) {
                            Ok(updated) => format!("{{\"msg\": \"{}\"}}", (match updated { true => "updated", false => "created" }) ),
                            Err(e) => format!("{{ \"err\": \"data structure error\", \"exception\": {:?}}}", e)
                        };

                    }

                }else if path[0] == "query" {
                    if path.len() == 2 {
                        let b = match hyper::body::to_bytes(_req).await {
                            Ok(b) => b,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error receiving request\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        let json_body: Value = match serde_json::from_slice(b.as_ref()) {
                            Ok(v) => v,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error parsing json\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        let offset = match params.get("offset") {
                            Some(n) => {
                                match n.parse::<u64>() {
                                    Ok(num) => num,
                                    Err(e) => {
                                        response = format!("{{ \"err\": \"error parsing offset url parameter\", \"exception\": {:?}}}", e);
                                        return Ok(Response::new(response.into()));
                                    }
                                }
                            },
                            None => 0
                        };

                        let limit = match params.get("limit") {
                            Some(n) => {
                                match n.parse::<u64>() {
                                    Ok(num) => num,
                                    Err(e) => {
                                        response = format!("{{ \"err\": \"error parsing limit url parameter\", \"exception\": {:?}}}", e);
                                        return Ok(Response::new(response.into()));
                                    }
                                }
                            },
                            None => 10
                        };

                        let model = Model::get(&conn, path[1]);

                        response = match query::select_query(&conn, &model, limit, offset, json_body) {
                            Ok(data) => data,
                            Err(e) => e
                        };
                    }
                }
            }
        }
        _=> {}
    };

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .status(StatusCode::OK)
        .body(response.into()).unwrap())
}

#[tokio::main]
async fn main() {
    log::set_logger(&SomeLogger)
        .map(|()| log::set_max_level(LevelFilter::Info)).unwrap();

    let addr = ([127, 0, 0, 1], 3000).into();

    let manager = SqliteConnectionManager::file("data.db");
    let pool = r2d2::Pool::new(manager).unwrap();

    let make_service = make_service_fn(move |_| {
        let pool = pool.clone();

        async move {
            Ok::<_, Error>(service_fn(move |_req| {
                let conn = pool.get().unwrap();
                service(conn, _req)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}