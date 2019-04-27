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
use std::thread;
use structopt::StructOpt;

use futures::{future, Future};

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
enum Cargo {
    #[structopt(name = "docserver")]
    Docserver {
        #[structopt(short = "p", long = "port", default_value = "4000")]
        port: u16,

        #[structopt(long, short, allow_hyphen_values = true)]
        /// The arguments that will be sent to `cargo doc` when recompiling the docs.
        recompile_args: Option<String>,
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
        let package_name = &meta.packages[0].name;
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

fn setup_recompilation_watcher(recompile_args: Option<String>) {
    thread::spawn(move || {
        let recompile_args = into_command_args(&recompile_args);
        let mut input = String::new();

        println!("Press ENTER to recompile the docs");
        loop {
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    compile_docs(&recompile_args);
                }
                Err(error) => eprintln!("Error reading from stdin: {}", error),
            }
        }
    });
}

fn into_command_args(args: &Option<String>) -> Vec<&str> {
    match args.as_ref() {
        Some(args) => args.split(' ').collect(),
        None => Vec::new(),
    }
}

fn compile_docs(recompile_args: &[&str]) {
    use std::process::Command;

    println!(
        "Compiling docs with `cargo doc {}`",
        recompile_args.join(" ")
    );

    Command::new("cargo")
        .args(&["doc"])
        .args(recompile_args)
        .spawn()
        .expect("failed to compile docs");
}

fn main() {
    match Cargo::from_args() {
        Cargo::Docserver {
            port,
            recompile_args,
        } => {
            let addr = ([0, 0, 0, 0], port).into();
            let svc = || service_fn(serve_docs);
            let server = Server::bind(&addr)
                .serve(svc)
                .map_err(|e| eprintln!("server error {}", e));

            println!("Listening on http://{}", addr);
            setup_recompilation_watcher(recompile_args);
            hyper::rt::run(server);
        }
    }
}
