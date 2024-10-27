use std::{collections::HashMap, fmt::Display, str::FromStr};
use reqwest;
use serde_json::Value;
use url::{Url, ParseError};
use structopt::StructOpt;

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
}

fn main() {
    let opt = Opt::from_args();

    println!("Requesting URL: {}", &opt.url);
    println!("Method: {}", opt.method);

    if let Some(data) = &opt.data {
        println!("Data: {}", data);
    }

    let url = match Url::parse(&opt.url) {
        Ok(url) => {
            // Restrict to HTTP and HTTPS
            if url.scheme() != "http" && url.scheme() != "https" {
                println!("Error: The URL does not have a valid base protocol.");
                return;
            }

            url
        },
        Err(e) => {
            match e {
                ParseError::RelativeUrlWithoutBase | ParseError::RelativeUrlWithCannotBeABaseBase | ParseError::SetHostOnCannotBeABaseUrl => println!("Error: The URL does not have a valid base protocol."),
                ParseError::InvalidIpv4Address | ParseError::InvalidIpv6Address => println!("Error: The URL contains an invalid IPv6 address."),
                ParseError::InvalidPort => println!("Error: The URL contains an invalid port number."),
                _ => println!("Error: {e}"),
            }

            return;
        }
    };

    match make_request(opt) {
        Ok(resp) => {
            if !resp.status().is_success() {
                println!("Error: Request failed with status code: {}.", resp.status().as_u16());
                return;
            }

            let body = resp.text().unwrap();

            // Check if response is JSON
            let v: Value = match serde_json::from_str(&body) {
                Ok(json) => json,
                Err(e) => Value::Null,
            };

            if v.is_null() {
                println!("Response body:");
                println!("{}", body);
            } else {
                println!("Response JSON:");
                println!("{:#?}", v);
            }
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
    let resp = match opt.method {
        Method::GET => {
            reqwest::blocking::get(&opt.url)?
        },
        Method::POST => {
            let client = reqwest::blocking::Client::new();
            let data = opt.data.unwrap();
            let params = parse_params(&data);

            client.post(&opt.url)
                .form(&params)
                .send()?
        }
    };

    Ok(resp)
}

fn parse_params(data: &str) -> HashMap<&str, &str> {
    let mut params = HashMap::new();

    for param in data.split('&') {
        let parts: Vec<&str> = param.split('=').collect();

        params.insert(parts[0], parts[1]);
    }

    params
}
