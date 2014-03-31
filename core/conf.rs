// See LICENSE file for copyright and license details.

use serialize::{Decodable, json};
use core::misc::read_file;

pub struct Config {
    priv json: ~json::Object,
}

fn decode<A: Decodable<json::Decoder, json::Error>>(json_obj: json::Json) -> A {
    let mut decoder = json::Decoder::new(json_obj);
    let decoded: A = Decodable::decode(&mut decoder).unwrap();
    decoded
}

impl Config {
    pub fn new(path: &str) -> Config {
        let path = Path::new(path);
        let json = match json::from_str(read_file(&path)) {
            Ok(json::Object(obj)) => obj,
            _ => fail!("Config error"),
        };
        Config {
            json: json,
        }
    }

    pub fn get<A: Decodable<json::Decoder, json::Error>>(&self, name: &str) -> A {
        let owned_name_str = name.into_owned();
        decode(match self.json.find(&owned_name_str) {
            Some(val) => val.clone(),
            None => fail!("No field '{}", name),
        })
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
