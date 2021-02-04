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
use crate::ui::build_html_footer;

async fn service(conn: PooledConnection<SqliteConnectionManager>, _req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = String::new();
    let mut content_type = "application/json";
    let path_str = _req.uri().path().to_string();
    let path: Vec<&str> = path_str.split("/").collect();

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
            if path.len() > 1 {
                if path[1] == "api" {
                    if path.len() == 3 {
                        //model
                        let model = Model::get(&conn, path[2]);
                        response = String::from('{');
                        for r in model.fields {
                            response.push_str(format!("\"{}\": \"{}\",", r.0, r.1).as_str());
                        }
                        response.pop();
                        response.push('}');

                    }else if path.len() == 4 {
                        //instance
                        let id = match path[3].parse::<u64>() {
                            Ok(v) => v,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error parsing id\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        let instance = Instance::get(conn, path[2], id);
                        response = match instance {
                            Some(inst) => inst.json,
                            None => "null".to_string()
                        };
                    }
                }else if path[1] == "ui" {

                    response = ui::build_html_header();
                    content_type = "text/html";

                    if path.len() == 3 {
                        let model = Model::get(&conn, path[2]);
                        let html = ui::build_model_editor(&model);
                        response.push_str( html.as_str() );

                    }else {
                        let models = Model::get_all(&conn);

                        for m in models {
                            let html = ui::build_model_editor(&m);
                            response.push_str( html.as_str() );
                        }
                    }

                    response.push_str(ui::build_html_footer().as_str());
                }
            }
        },
        &Method::DELETE => {
            if path.len() > 2 {
                if path[1] == "api" {
                    if path.len() == 3 {
                        //model
                        response = match Model::delete(conn, path[2]) {
                            Ok(_) => "{\"msg\": \"deleted\"}".to_string(),
                            Err(e) => e
                        };

                    }else if path.len() == 4 {
                        //instance
                        let id = match path[3].parse::<u64>() {
                            Ok(v) => v,
                            Err(e) => {
                                response = format!("{{ \"err\": \"error parsing id\", \"exception\": {:?}}}", e);
                                return Ok(Response::new(response.into()));
                            }
                        };

                        response = match Instance::delete(conn, path[2], id) {
                            Ok(_) => "{\"msg\": \"deleted\"}".to_string(),
                            Err(e) => e
                        };
                    }
                }
            }
        },
        &Method::PUT => {
            if path.len() > 2 {
                if path[1] == "api" {
                    if path.len() == 3 {
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

                        response = match Instance::create(conn, path[2], json_body) {
                            Ok(_) => format!("{{\"msg\": \"created\"}}"),
                            Err(e) => format!("{{ \"err\": \"data structure error\", \"exception\": {:?}}}", e)
                        };

                    }else if path.len() == 4 {
                        //update instance
                        response = String::from("instance update not implemented");

                    }
                }
            }
        },
        &Method::POST => {
            if path.len() > 2 {
                if path[1] == "api" {
                    if path.len() == 3 {
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

                        response = match Model::update(conn, json_body, path[2]) {
                            Ok(updated) => format!("{{\"msg\": \"{}\"}}", (match updated { true => "updated", false => "created" }) ),
                            Err(e) => format!("{{ \"err\": \"data structure error\", \"exception\": {:?}}}", e)
                        };

                    }

                }else if path[1] == "query" {
                    if path.len() == 3 {
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

                        let model = Model::get(&conn, path[2]);

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