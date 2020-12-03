use std::path::Path;
use rocket::response::Responder;
use rocket::*;
use rocket::http::hyper::header::ByteRangeSpec;
use rocket::http::hyper::header::Range;
use rocket::response::Body;        
use std::str::FromStr;
use std::io::SeekFrom;
use rocket::http::ContentType;
use std::fs::File;
use rocket::http::Status;
use std::io::Seek;
use std::io::Read;


pub struct RangedFile(pub Box<Path>);

impl<'r> Responder<'r> for RangedFile {
    fn respond_to(self, request: &Request) -> response::Result<'r> {


        let range = request.headers().get("Range").next();

        let mut response = Response::build();
        response.raw_header("Accept-Ranges", "bytes");
        response.header(ContentType::MP4);
        if let Some(range) = range {
            let range = Range::from_str(range).unwrap();
            println!("range = {:?}", range);
            if let Range::Bytes(ranges) = range {
                if ranges.len() != 1 {
                    return Err(Status::RangeNotSatisfiable);
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

                    response.raw_header(
                        "Content-Range",
                        format!("bytes {}-{}/{}", start, end - 1, end),
                    );
                    println!("Setting body");
                    response.raw_body(Body::Sized(file, length));
                } else if let ByteRangeSpec::FromTo(start, end) = range_spec {
                    let mut file: File = File::open(self.0).unwrap();

                    let file_end = file.seek(SeekFrom::End(0)).unwrap();
                    file.seek(SeekFrom::Start(*start)).unwrap();
                    let length = end - *start + 1;

                    if *start != 0 {
                        response.status(Status::PartialContent);
                    }
                    let ranged_file = file.take(length);

                    response.raw_header(
                        "Content-Range",
                        format!("bytes {}-{}/{}", start, end, file_end),
                    );

                    println!("Setting body");
                    response.raw_body(Body::Sized(ranged_file, length));
                } else {
                    return Err(Status::RangeNotSatisfiable);
                }
            } else {
                return Err(Status::RangeNotSatisfiable);
            }
        } else {
            response.sized_body(File::open(self.0).unwrap());
        }
        println!("answering with ok");
        response.ok()
    }
}
