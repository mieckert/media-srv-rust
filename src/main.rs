#![feature(proc_macro_hygiene, decl_macro)]

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate handlebars;

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
use dir::Entry;
use dir::FileType;
use dir::DIRECTORY_ICON;
mod ranged_file;
use ranged_file::*;
mod to_url;

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(dir: ""))
}

#[derive(Serialize)]
struct DirHbsContext {
    dir: String,
    entries: Vec<Entry>,
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
        .map(|m| Entry {
            name: m.mount_point.to_string_lossy().into_owned(),
            link_url: format!("/dir/{}", &m.mount_point.to_string_lossy()),
            download_url: "".to_string(),
            filetype: FileType::Directory,
            icon: DIRECTORY_ICON
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

    match dir::read_dir(real_dir, dir.clone()) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(status::Custom(
            Status::NotFound,
            "Directory not found".to_string(),
        )),
        Err(e) => Err(status::Custom(
            Status::InternalServerError,
            format!("Internal Server Error: {:?}", e),
        )),
        Ok(entries) => {
            let dir = dir.to_string_lossy().into_owned();

            let mut entries: Vec<Entry> = entries.into_iter().filter(|entry| {
                match entry.filetype {
                    FileType::Directory => true, 
                    FileType::Video => true,
                    _ => false
                }
            }).collect();

            entries.sort_by(|a,b| {
                match (&a.filetype, &b.filetype) {
                    (FileType::Directory, FileType::Directory) => a.name.cmp(&b.name),
                    (FileType::Directory, _) => std::cmp::Ordering::Less,
                    (_, FileType::Directory) => std::cmp::Ordering::Greater,
                    (_,_) => a.name.cmp(&b.name)                    
                }
            });

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

handlebars_helper!(hbs_helper_is_video: |filetype: str| {
    //println!("hbs_helper_is_video called with '{}'", &filetype);
    filetype == "Video"
});

fn main() {
    let r = rocket::ignite();
    println!("Rocket launch config: {:?}", r.config());
    
    let template_fairing = Template::custom(|engines| {
        engines.handlebars.register_helper("is-video", Box::new(hbs_helper_is_video));
    });

    r.attach(ConfigFairing::new())
        //.attach(Template::fairing())
        .attach(template_fairing)
        .mount("/static", StaticFiles::from("static"))
        .mount("/", routes![index, dir_root, dir, watch, files])
        //.register(catchers![not_found, bad_request])
        .launch();
}
