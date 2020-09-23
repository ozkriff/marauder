// See LICENSE file for copyright and license details.

use crate::core::misc::read_file;
use std::path::Path;

pub struct Config {
    json: serde_json::Value,
}

// fn decode<A: Decodable<json::Decoder, json::DecoderError>>(json_obj: json::Json) -> A {
//     let mut decoder = json::Decoder::new(json_obj);
//     let decoded: A = Decodable::decode(&mut decoder).unwrap();
//     decoded
// }

impl Config {
    pub fn new(path: &Path) -> Config {
        // set_error_context!("parsing config", path.as_str().unwrap());
        let json = match serde_json::from_str(&read_file(path)) {
            Ok(res) => res,
            Err(msg) => panic!("Config parsing error: {}", msg),
            // some_error => panic!("Unknown config parsing error: {}", some_error),
        };
        Config { json }
    }

    pub fn get(&self, name: &str) -> &serde_json::Value {
        &self.json[name]
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
