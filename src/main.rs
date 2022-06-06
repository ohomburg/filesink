use bytes::{Buf, Bytes};
use constant_time_eq::constant_time_eq;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::process::Command;
use warp::http::Response as HResponse;
use warp::reply::Response;
use warp::Filter;

#[derive(Clone, Deserialize)]
struct Endpoint {
    cmd: String,
    auth: String,
    target: Option<String>,
}

fn respond(status: u16, msg: &'static str) -> Response {
    HResponse::builder()
        .status(status)
        .body(msg.into())
        .unwrap()
}

fn file_sink(
    name: String,
    auth: String,
    body: Bytes,
    config: &HashMap<String, Endpoint>,
) -> Result<Response, Response> {
    let ep = config.get(&name).ok_or(respond(404, "Not Found"))?;

    if !constant_time_eq(ep.auth.as_bytes(), auth.as_bytes()) {
        return Err(respond(401, "Unauthorized"));
    }

    let path = ep.target.as_ref().map(|s| &**s).unwrap_or(&name);
    let mut file = File::create(path).map_err(|e| {
        eprintln!("File creation failed: {:?}", e);
        respond(500, "file creation failed")
    })?;
    std::io::copy(&mut body.reader(), &mut file).map_err(|e| {
        eprintln!("Copy failed: {:?}", e);
        respond(500, "copy failed")
    })?;

    println!("Received valid PUT for {}", name);
    println!("Running {}", ep.cmd);
    let status = Command::new("sh")
        .arg("-c")
        .arg(&ep.cmd)
        .status()
        .map_err(|_| respond(500, "command failed"))?;
    if status.success() {
        Ok(respond(200, ""))
    } else {
        Err(HResponse::builder()
            .status(500)
            .body(format!("Status code {:?}", status.code()).into())
            .unwrap())
    }
}

fn file_sink_wrapper(
    file: String,
    auth: String,
    body: Bytes,
    config: &HashMap<String, Endpoint>,
) -> Response {
    file_sink(file, auth, body, config).unwrap_or_else(|x| x)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_file = match std::env::args_os().nth(1) {
        Some(x) => x,
        None => {
            eprintln!("Config file argument required.");
            std::process::exit(1);
        }
    };

    let config_file = std::fs::File::open(config_file)?;
    let config: Box<HashMap<String, Endpoint>> = Box::new(serde_yaml::from_reader(config_file)?);

    let route = warp::put()
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::query::raw())
        .and(warp::body::bytes())
        .map(move |f, a, b| file_sink_wrapper(f, a, b, &*config));

    warp::serve(route).run(([0, 0, 0, 0], 8228)).await;
    Ok(())
}
