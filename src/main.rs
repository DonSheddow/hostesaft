extern crate reqwest;
extern crate clap;

use reqwest::{Client, RedirectPolicy};
use reqwest::header::Host;
use clap::{App, Arg};
use std::io::{self, BufRead};
use std::fs::File;
use std::io::Error;

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
        .build().unwrap();

    for host in hosts {
        let res = get_url(&client, url, host.clone()).unwrap();
        println!("{}: {}", host, res);
    }

}


fn get_url(client: &Client, url: &str, host: String) -> Result<String, reqwest::Error> {
    let resp = client.get(url)
        .header(Host::new(host, None))
        .send()?;

    Ok(resp.status().to_string())
}
