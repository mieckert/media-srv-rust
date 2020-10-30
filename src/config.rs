use rocket::Rocket;
use rocket::fairing::{Fairing, Info, Kind};
use core::result::Result;

pub struct Config {
    pub root_dir: String,
    pub base_url: String
}

#[derive(Default)]
pub struct ConfigFairing;

impl ConfigFairing {
    pub fn new() -> Self {
        Self
    }
}

impl Fairing for ConfigFairing {
    fn info(&self) -> Info {
        Info {
            name: "Configuration Fairing",
            kind: Kind::Attach
        }
    }
    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let root_dir = rocket.config()
            .get_string("root_dir")
            .expect("root_dir not configured in Rocket.toml");                
        let base_url = rocket.config()
            .get_string("base_url")
            .expect("base_url not configured in Rocket.toml");                

        Ok(rocket.manage(Config { root_dir, base_url }))
    }
}