extern crate reqwest;
extern crate clap;

use reqwest::{Client, RedirectPolicy, StatusCode};
use reqwest::header::{self, Headers, Host};
use clap::{App, Arg};
use std::io::{self, BufRead};
use std::fs::File;
use std::io::Error;
use std::fmt;


struct Response {
    body: String,
    status: StatusCode,
    headers: Headers,
}


impl Response {
    fn new(mut resp: reqwest::Response) -> Self {
        Response {
            body: resp.text().unwrap_or("".to_string()),
            status: resp.status(),
            headers: resp.headers().clone(),
        }
    }

    fn sentinel() -> Self {
        Response {
            body: "".to_string(),
            status: StatusCode::Unregistered(0),
            headers: Headers::new(),
        }
    }

    fn is_equal_to(&self, other: &Response) -> bool {
        if self.status != other.status {
            return false;
        }
        if self.status.is_redirection() {
            return self.headers.get::<header::Location>().unwrap()
                == other.headers.get::<header::Location>().unwrap();
        }

        self.body == other.body
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.status.is_redirection() {
            write!(f, "{} -> {}", self.status, self.headers.get::<header::Location>().unwrap())
        }
        else {
            write!(f, "{}", self.status)
        }
    }
}


fn main() {
    let matches = App::new("Hostesaft")
                          .version("1.0")
                          .author("Sigurd Kolltveit <sigurd.kolltveit@gmx.com>")
                          .arg(Arg::with_name("hosts")
                               .required(true)
                               .index(1))
                          .arg(Arg::with_name("url")
                               .required(true)
                               .index(2))
                          .get_matches();

    let url = matches.value_of("url").unwrap();
    let hosts_file = matches.value_of("hosts").unwrap();
    let hosts = io::BufReader::new(File::open(hosts_file).unwrap())
        .lines()
        .collect::<Result<Vec<String>, Error>>()
        .unwrap();

    println!("Testing hostnames on '{}':\n", url);

    let client = reqwest::Client::builder()
        .redirect(RedirectPolicy::none())
        .danger_disable_hostname_verification()
        .build().unwrap();

    let default_resp = get_url(&client, url, "non-existing.host".to_string()).unwrap();

    for host in hosts {
        let trivial_resp = get_url(&client, &format!("https://{}", &host), host.clone())
            .or_else(|_| get_url(&client, &format!("http://{}", &host), host.clone()))
            .unwrap_or_else(|_| Response::sentinel());

        let resp = get_url(&client, url, host.clone()).unwrap();
        let alert = if !resp.is_equal_to(&default_resp) && !resp.is_equal_to(&trivial_resp) {
            "[!!] "
        }
        else {
            ""
        };
        print!("{}{}\n\t", alert, host);
        println!("{}", resp);
    }

}


fn get_url(client: &Client, url: &str, host: String) -> reqwest::Result<Response> {
    let resp = client.get(url)
        .header(Host::new(host, None))
        .send()?;

    Ok(Response::new(resp))
}
