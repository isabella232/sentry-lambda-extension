use anyhow::{ensure, Result};
use relay_config::{Config, OverridableConfig};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use sentry_types::Dsn;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time;
use std::fmt::Write;

extern crate libc;
use libc::{raise, SIGTERM};

const EXTENSION_NAME: &str = "sentry-lambda-extension";
const EXTENSION_NAME_HEADER: &str = "Lambda-Extension-Name";
const EXTENSION_ID_HEADER: &str = "Lambda-Extension-Identifier";
const SHUTDOWN_TIMEOUT: u64 = 2;

fn base_url() -> Result<String, env::VarError> {
    Ok(format!(
        "http://{}/2020-01-01/extension",
        env::var("AWS_LAMBDA_RUNTIME_API")?
    ))
}

fn upstream_url() -> Option<String> {
    if let Ok(dsn) = env::var("SENTRY_DSN") {
        if let Ok(dsn) = dsn.parse::<Dsn>() {
            let mut buf = format!("{}://{}", dsn.scheme(), dsn.host());
            if dsn.port() != dsn.scheme().default_port() {
                write!(&mut buf, ":{}", dsn.port()).ok()?;
            }
            return Some(buf);
        }
    }
    None
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Tracing {
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "UPPERCASE", tag = "eventType")]
enum NextEventResponse {
    #[serde(rename_all = "camelCase")]
    Invoke {
        deadline_ms: u64,
        request_id: String,
        invoked_function_arn: String,
        tracing: Tracing,
    },
    #[serde(rename_all = "camelCase")]
    Shutdown {
        shutdown_reason: String,
        deadline_ms: u64,
    },
}

fn next_event(client: &reqwest::blocking::Client, ext_id: &str) -> Result<NextEventResponse> {
    let url = format!("{}/event/next", base_url()?);
    Ok(client
        .get(&url)
        .header(EXTENSION_ID_HEADER, ext_id)
        .send()?
        .json()?)
}

#[derive(Debug)]
struct RegisterResponse {
    pub extension_id: String,
}

fn register(client: &reqwest::blocking::Client) -> Result<RegisterResponse> {
    let mut map = HashMap::new();
    map.insert("events", vec!["INVOKE", "SHUTDOWN"]);
    let url = format!("{}/register", base_url()?);
    let res = client
        .post(&url)
        .header(EXTENSION_NAME_HEADER, EXTENSION_NAME)
        .json(&map)
        .send()?;

    ensure!(
        res.status() == StatusCode::OK,
        "Unable to register extension",
    );

    let ext_id = res.headers().get(EXTENSION_ID_HEADER).unwrap().to_str()?;

    Ok(RegisterResponse {
        extension_id: ext_id.into(),
    })
}

fn make_config() -> Result<Config> {
    let mut config = Config::default();

    // TODO(neel): add shutdown_timeout later
    let overrides = OverridableConfig {
        mode: Some("proxy".to_string()),
        shutdown_timeout: Some(SHUTDOWN_TIMEOUT.to_string()),
        upstream: upstream_url(),
        ..Default::default()
    };

    config.apply_override(overrides).map_err(failure::Fail::compat)?;
    Ok(config)
}

fn start_relay() -> Result<()> {
    // Run relay in background
    println!("Starting Sentry `relay` in background...");

    let config = make_config()?;
    relay_log::init(config.logging(), config.sentry());
    std::thread::spawn(|| relay_server::run(config));

    Ok(())
}

fn main() -> Result<()> {
    //Register the Lambda extension
    println!("Starting Sentry Lambda Extension...");
    start_relay()?;
    let client = Client::builder().timeout(None).build()?;
    let response = register(&client)?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst))?;

    while running.load(Ordering::SeqCst) {

        println!("Waiting for event...");
        let evt = next_event(&client, &response.extension_id);

        match evt {
            Ok(evt) => match evt {
                NextEventResponse::Invoke {
                    request_id,
                    deadline_ms,
                    ..
                } => {
                    println!("Invoke event {}; deadline: {}", request_id, deadline_ms);
                }
                NextEventResponse::Shutdown {
                    shutdown_reason, ..
                } => {
                    println!("Exiting: {}", shutdown_reason);
                    unsafe {
                        raise(SIGTERM);
                    }
                    std::thread::sleep(time::Duration::from_secs(SHUTDOWN_TIMEOUT));
                    return Ok(());
                }
            },
            Err(err) => {
                eprintln!("Error: {:?}", err);
                println!("Exiting");
                return Err(err);
            }
        }
    }

    Ok(())
}
