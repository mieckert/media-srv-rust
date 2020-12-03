use core::result::Result;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

pub struct Config {
    pub base_url: String,
    pub mounts: Vec<Mount>,
}

pub struct Mount {
    pub mount_point: PathBuf,
    pub local_dir: PathBuf,
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
            kind: Kind::Attach,
        }
    }
    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let base_url = rocket
            .config()
            .get_string("base_url")
            .expect("base_url not configured in Rocket.toml");

        let mounts: HashMap<String, String> =
            toml::from_str(&read_to_string("mount.toml").expect("mount.toml could not be read"))
                .expect("Data format in mount.toml incorrect");

        let mounts: Vec<Mount> = mounts
            .iter()
            .map(|(k, v)| Mount {
                mount_point: PathBuf::from(k),
                local_dir: PathBuf::from(v),
            })
            .collect();

        Ok(rocket.manage(Config {
            base_url,
            mounts,
        }))
    }
}
