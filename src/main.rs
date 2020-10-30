#![feature(proc_macro_hygiene, decl_macro)]

use std::path::Path;
use std::path::PathBuf;
use std::io;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[macro_use]
extern crate rocket;

use rocket::request::Request;
use rocket::response;
use rocket::response::Response;
use rocket::response::Responder;
use rocket::http::ContentType;
use rocket::State;
use rocket::http::Status;
use rocket::response::status;
use rocket::response::Redirect;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use serde::Serialize;

mod config;
use config::{Config, ConfigFairing};
mod dir;


#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(dir: ""))
}

#[get("/dir")]
fn dir_root(cfg: State<Config>) -> Result<Template, status::Custom<String>> {
    dir(PathBuf::from(""), cfg)
}

#[get("/dir/<dir..>")]
fn dir(dir: PathBuf, cfg: State<Config>) -> Result<Template, status::Custom<String>> {
    #[derive(Serialize)]
    struct Context {
        dir: String,
        subdirs: Vec<String>,
        files: Vec<String>
    }

    if !dir.is_relative() {
        return Err(status::Custom( Status::BadRequest, "Bad Request: directory is not relative".to_string() ))
    }

    let mut real_dir = PathBuf::from(&cfg.root_dir);        
    real_dir.push(&dir);    
    let dir = dir.to_string_lossy().into_owned();    

    match dir::read_dir(real_dir) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            Err(status::Custom(
                Status::NotFound, "Directory not found".to_string()
            ))
        },
        Err(e) => {
            Err(status::Custom(
                Status::InternalServerError, format!("Internal Server Error: {:?}", e)
            ))
        },
        Ok((subdirs, files)) => {
            let context = Context { dir, subdirs, files };
            Ok(Template::render("dir", &context))        
        }
    }
}

#[get("/watch/<file_path..>")]
fn watch(file_path: PathBuf) -> Result<Template, status::Custom<String>> {
    let mut context: HashMap<String,String> = HashMap::new();
    context.insert("file_path".to_string(), file_path.to_string_lossy().into_owned());
    Ok(Template::render("watch", &context))        
}

struct RangedFile(Box<Path>);


impl<'r> Responder<'r> for RangedFile {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        use rocket::http::hyper::header::Range;
        use rocket::http::hyper::header::ByteRangeSpec;
        use rocket::response::Body;
        use std::str::FromStr;
        use std::io::Seek;

        use std::io::SeekFrom;

        let range = request.headers().get("Range").next();        

        let mut response = Response::build();
        response.raw_header("Accept-Ranges", "bytes");
        response.header(ContentType::MP4);
    
        if let Some(range) = range {
            let range = Range::from_str(range).unwrap();
            println!("range = {:?}", range);
            if let Range::Bytes(ranges) = range {
                if ranges.len() != 1 {
                    return Err( Status::RangeNotSatisfiable )
                }
                let range_spec = ranges.first().unwrap();
                if let ByteRangeSpec::AllFrom(start) = range_spec {
                    let mut file = File::open(self.0).unwrap();

                    let end = file.seek(SeekFrom::End(0)).unwrap();                    
                    file.seek(SeekFrom::Start(*start)).unwrap();
                    let length = end - *start;

                    if *start != 0 {
                        response.status(Status::PartialContent);
                    }
                    
                    response.raw_header("Content-Range", format!("bytes {}-{}/{}", start, end-1, end));                    

                    println!("Setting body");
                    response.raw_body(Body::Sized(file, length));

                }
                else if let ByteRangeSpec::FromTo(start,end) = range_spec {
                    let mut file: File = File::open(self.0).unwrap();
                    
                    let file_end = file.seek(SeekFrom::End(0)).unwrap();                    
                    file.seek(SeekFrom::Start(*start)).unwrap();
                    let length = end - *start+1;

                    if *start != 0 {
                        response.status(Status::PartialContent);
                    }
                    
                    let ranged_file = file.take(length);

                    response.raw_header("Content-Range", format!("bytes {}-{}/{}", start, end, file_end));                    

                    println!("Setting body");
                    response.raw_body(Body::Sized(ranged_file, length));

                }                
                else {
                    return Err( Status::RangeNotSatisfiable )
                }
            }
            else {
                return Err( Status::RangeNotSatisfiable )
            }
        }
        else {
            response.sized_body(File::open(self.0).unwrap());
        }
        
        println!("answering with ok");
        response.ok()
    }
}

#[get("/files/<file_path..>")]
fn files(file_path: PathBuf, cfg: State<Config>) -> Result<RangedFile, status::Custom<String>> {  
    let mut real_path = PathBuf::from(&cfg.root_dir);        
    real_path.push(&file_path);    
    
    Ok( RangedFile( real_path.into_boxed_path() ) )
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