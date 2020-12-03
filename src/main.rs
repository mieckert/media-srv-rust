#![feature(proc_macro_hygiene, decl_macro)]

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::response::status;
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use serde::Serialize;

mod config;
use config::{Config, ConfigFairing};
mod dir;
mod ranged_file;
use ranged_file::*;

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(dir: ""))
}

#[derive(Serialize)]
struct DirHbsContext {
    dir: String,
    entries: Vec<DirHbsContextEntry>,
}

#[derive(Serialize)]
struct DirHbsContextEntry {
    label: String,
    link: String,
    icon: String,
}

#[get("/dir")]
fn dir_root(cfg: State<Config>) -> Result<Template, status::Custom<String>> {
    /*
    let subdirs: Vec<String> = cfg.mounts.iter().flat_map(|m|
        if let Some(Normal( first )) = m.mount_point.components().next() {
            Some(first.to_string_lossy().into_owned())
        }
        else {
            None
        }
    ).collect();
    */
    let entries = cfg
        .mounts
        .iter()
        .map(|m| DirHbsContextEntry {
            label: m.mount_point.to_string_lossy().into_owned(),
            link: format!("/dir/{}", &m.mount_point.to_string_lossy()),
            icon: String::from("folder"),
        })
        .collect();

    // TODO: fix "///"" issue in template
    let context = DirHbsContext {
        dir: String::from("/"),
        entries,
    };
    Ok(Template::render("dir", &context))
}

fn real_dir(dir: &PathBuf, cfg: State<Config>) -> Result<PathBuf, status::Custom<String>> {
    if !dir.is_relative() {
        return Err(status::Custom(
            Status::BadRequest,
            "Path is not relative".to_string(),
        ));
    }

    let m = cfg.mounts.iter().find(|m| dir.starts_with(&m.mount_point));
    if m.is_none() {
        return Err(status::Custom(
            Status::NotFound,
            "Path does not start with a mounted directory".to_string(),
        ));
    }
    let m = m.unwrap();
    let dir = dir.strip_prefix(&m.mount_point);
    if dir.is_err() {
        return Err(status::Custom(
            Status::InternalServerError,
            "Could not strip prefix from path".to_string(),
        ));
    }
    let dir = dir.unwrap().to_owned();
    let mut real_dir = PathBuf::from(&m.local_dir);
    real_dir.push(&dir);

    Ok(real_dir)
}

#[get("/dir/<dir..>")]
fn dir(dir: PathBuf, cfg: State<Config>) -> Result<Template, status::Custom<String>> {
    let real_dir = real_dir(&dir, cfg)?;

    match dir::read_dir(real_dir) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(status::Custom(
            Status::NotFound,
            "Directory not found".to_string(),
        )),
        Err(e) => Err(status::Custom(
            Status::InternalServerError,
            format!("Internal Server Error: {:?}", e),
        )),
        Ok((subdirs, files)) => {
            let dir = dir.to_string_lossy().into_owned();

            let entries = subdirs
                .into_iter()
                .map(|d| DirHbsContextEntry {
                    link: format!("/dir/{}/{}", &dir, &d),
                    label: d,
                    icon: String::from("folder"),
                })
                .chain(files.into_iter().map(|f| DirHbsContextEntry {
                    link: format!("/watch/{}/{}", &dir, &f),
                    label: f,
                    icon: String::from("file-earmark"),
                }))
                .collect();
            let context = DirHbsContext { dir, entries };
            Ok(Template::render("dir", &context))
        }
    }
}

#[get("/watch/<file_path..>")]
fn watch(file_path: PathBuf) -> Result<Template, status::Custom<String>> {
    let mut context: HashMap<String, String> = HashMap::new();
    context.insert(
        "file_path".to_string(),
        file_path.to_string_lossy().into_owned(),
    );
    context.insert(
        "file_name".to_string(),
        file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
    );
    Ok(Template::render("watch", &context))
}

#[get("/files/<file_path..>")]
fn files(file_path: PathBuf, cfg: State<Config>) -> Result<RangedFile, status::Custom<String>> {
    let real_path = real_dir(&file_path, cfg)?;

    Ok(RangedFile(real_path.into_boxed_path()))
}

fn main() {
    let r = rocket::ignite();
    println!("Rocket launch config: {:?}", r.config());

    r.attach(ConfigFairing::new())
        .attach(Template::fairing())
        .mount("/static", StaticFiles::from("static"))
        .mount("/", routes![index, dir_root, dir, watch, files])
        //.register(catchers![not_found, bad_request])
        .launch();
}
