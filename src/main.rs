use reqwest;
use serde_json::Value;
use std::{collections::HashMap, fmt::Display, str::FromStr};
use structopt::StructOpt;
use url::{ParseError, Url};

enum Method {
    GET,
    POST,
}

#[derive(Debug)]
enum MethodError {
    InvalidMethod,
}

impl Display for MethodError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid HTTP method")
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
        }
    }
}

impl FromStr for Method {
    type Err = MethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            _ => Err(MethodError::InvalidMethod),
        }
    }
}

#[derive(StructOpt)]
#[structopt(name = "curl")]
struct Opt {
    url: String,

    #[structopt(short)]
    data: Option<String>,

    #[structopt(short = "X", default_value = "GET")]
    method: Method,

    #[structopt(long)]
    json: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    println!("Requesting URL: {}", &opt.url);

    if let Some(json) = &opt.json {
        println!("Method: {}", Method::POST);
        println!("JSON: {}", json);
    } else {
        println!("Method: {}", opt.method);

        if let Some(data) = &opt.data {
            println!("Data: {}", data);
        }
    }

    match Url::parse(&opt.url) {
        Ok(url) => {
            // Restrict to HTTP and HTTPS
            if url.scheme() != "http" && url.scheme() != "https" {
                println!("Error: The URL does not have a valid base protocol.");
            }
        }
        Err(e) => match e {
            ParseError::RelativeUrlWithoutBase
            | ParseError::RelativeUrlWithCannotBeABaseBase
            | ParseError::SetHostOnCannotBeABaseUrl => {
                println!("Error: The URL does not have a valid base protocol.")
            }
            ParseError::InvalidIpv4Address => {
                println!("Error: The URL contains an invalid IPv4 address.")
            }
            ParseError::InvalidIpv6Address => {
                println!("Error: The URL contains an invalid IPv6 address.")
            }
            ParseError::InvalidPort => println!("Error: The URL contains an invalid port number."),
            _ => println!("Error: {e}"),
        },
    };

    match make_request(opt) {
        Ok(resp) => {
            if !resp.status().is_success() {
                println!(
                    "Error: Request failed with status code: {}.",
                    resp.status().as_u16()
                );
                return;
            }

            let body = resp.text().unwrap();

            // Check if response is JSON
            match serde_json::from_str::<Value>(&body) {
                Ok(json) => {
                    println!("Response body (JSON with sorted keys):");
                    println!("{:#}", json);
                }
                Err(_) => {
                    println!("Response body:");
                    println!("{}", body.trim());
                }
            };
        }
        Err(e) => {
            if e.is_timeout() || e.is_connect() {
                println!("Error: Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.");
                return;
            }
        }
    }
}

fn make_request(opt: Opt) -> Result<reqwest::blocking::Response, reqwest::Error> {
    // JSON request
    if let Some(json) = opt.json {
        let json: Value = match serde_json::from_str(&json) {
            Ok(json) => json,
            Err(e) => {
                panic!("Invalid JSON: {:#?}", e);
            }
        };

        let client = reqwest::blocking::Client::new();

        let resp = client.post(&opt.url).json(&json).send()?;

        return Ok(resp);
    }

    // Non-JSON request
    let resp = match opt.method {
        Method::GET => reqwest::blocking::get(&opt.url)?,
        Method::POST => {
            let client = reqwest::blocking::Client::new();
            let data = opt.data.unwrap();
            let params = parse_params(&data);

            client.post(&opt.url).form(&params).send()?
        }
    };

    Ok(resp)
}

fn parse_params(data: &str) -> HashMap<&str, &str> {
    let mut params = HashMap::new();

    for param in data.split('&') {
        let parts: Vec<&str> = param.split('=').collect();

        if parts.len() >= 2 {
            params.insert(parts[0], parts[1]);
        }
    }

    params
}
