extern crate structopt;

extern crate cargo_metadata;
extern crate hyper;

extern crate futures;
extern crate mime_guess;
extern crate tokio_fs;
extern crate tokio_io;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::io;
use std::path::{Path, PathBuf};
use std::str;
use structopt::StructOpt;

use futures::{future, Future};

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
enum Cargo {
    #[structopt(name = "docserver")]
    Docserver {
        #[structopt(short = "p", long = "port", default_value = "4000")]
        port: u16,
    },
}

type ResponseFuture = Box<Future<Item = Response<Body>, Error = io::Error> + Send>;

#[derive(Debug)]
struct CrateInfo {
    name: String,
    doc_path: PathBuf,
}

impl CrateInfo {
    fn parse() -> CrateInfo {
        let meta = cargo_metadata::metadata(None).unwrap();
        let package_name = &meta.packages[0].targets[0].name;
        let package_name_sanitized = str::replace(&package_name, "-", "_");
        let doc_path = Path::new(&meta.target_directory).join("doc");
        CrateInfo {
            name: package_name_sanitized.clone(),
            doc_path: doc_path.clone(),
        }
    }
}

#[test]
fn test_make_relative() {
    assert_eq!("foo/bar/baz", make_relative("/foo/bar/baz"));
    assert_eq!("foo/bar/baz", make_relative("///foo/bar/baz"));
}

#[test]
fn test_make_root_document() {
    assert_eq!("/foo/hello/index.html", make_index("/foo/hello"));
    assert_eq!("/foo/hello/index.html", make_index("/foo/hello/"));
    assert_eq!("/foo/hello.foo", make_index("/foo/hello.foo"));
}

fn make_index(path: &str) -> String {
    let sanitized_path = path.trim_end_matches("/");
    if sanitized_path.contains(".") {
        sanitized_path.to_string()
    } else {
        format!("{}/index.html", sanitized_path)
    }
}

fn make_relative(path: &str) -> String {
    path.trim_start_matches("/").to_string()
}

fn serve_docs(req: Request<Body>) -> ResponseFuture {
    let info = CrateInfo::parse();
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Box::new(future::ok(
            Response::builder()
                .status(302)
                .header("Location", format!("/{}/index.html", &info.name).as_str())
                .body(Body::empty())
                .unwrap(),
        )),
        (&Method::GET, path) => {
            let full_path = info.doc_path.join(&make_index(&make_relative(path)));
            let mime_type = format!("{}", mime_guess::guess_mime_type(&full_path));
            Box::new(
                tokio_fs::file::File::open(full_path)
                    .and_then(|file| {
                        let buf: Vec<u8> = Vec::new();
                        tokio_io::io::read_to_end(file, buf)
                            .and_then(move |item| {
                                Ok(Response::builder()
                                    .status(200)
                                    .header("Content-Type", mime_type.as_str())
                                    .body(item.1.into())
                                    .unwrap())
                            })
                            .or_else(|_| {
                                Ok(Response::builder()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(Body::empty())
                                    .unwrap())
                            })
                    })
                    .or_else(|_| {
                        Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body("not found".into())
                            .unwrap())
                    }),
            )
        }

        _ => not_found(),
    }
}

fn not_found() -> ResponseFuture {
    Box::new(future::ok(
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("not found".into())
            .unwrap(),
    ))
}

fn main() {
    match Cargo::from_args() {
        Cargo::Docserver { port } => {
            let addr = ([0, 0, 0, 0], port).into();
            let svc = || service_fn(serve_docs);
            let server = Server::bind(&addr)
                .serve(svc)
                .map_err(|e| eprintln!("server error {}", e));

            println!("Listening on http://{}", addr);
            hyper::rt::run(server);
        }
    }
}
