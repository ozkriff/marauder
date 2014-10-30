// See LICENSE file for copyright and license details.

use serialize::{Decodable, json};
use error_context;
use core::misc::read_file;

pub struct Config {
    json: json::JsonObject,
}

fn decode<A: Decodable<json::Decoder, json::DecoderError>>(json_obj: json::Json) -> A {
    let mut decoder = json::Decoder::new(json_obj);
    let decoded: A = Decodable::decode(&mut decoder).unwrap();
    decoded
}

impl Config {
    pub fn new(path: &Path) -> Config {
        set_error_context!("parsing config", path.as_str().unwrap());
        let json = match json::from_str(read_file(path).as_slice()) {
            Ok(json::Object(obj)) => obj,
            Err(msg) => panic!("Config parsing error: {}", msg),
            some_error => panic!("Unknown config parsing error: {}", some_error),
        };
        Config {
            json: json,
        }
    }

    pub fn get<A: Decodable<json::Decoder, json::DecoderError>>(&self, name: &str) -> A {
        let owned_name_str = name.into_string();
        decode(match self.json.find(&owned_name_str) {
            Some(val) => val.clone(),
            None => panic!("No field '{}", name),
        })
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
