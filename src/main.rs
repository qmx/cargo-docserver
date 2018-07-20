#[macro_use]
extern crate structopt;

extern crate hyper;

extern crate futures;
extern crate tokio_fs;
extern crate tokio_io;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::io;
use structopt::StructOpt;

use futures::{future, Future};

#[derive(Debug, StructOpt)]
#[structopt(name = "cargo")]
enum Cargo {
    #[structopt(name = "docserver")]
    Docserver {
        #[structopt(short = "p", long = "port", default_value = "4000")]
        port: u16,
    },
}

type ResponseFuture = Box<Future<Item = Response<Body>, Error = io::Error> + Send>;

fn serve_docs(req: Request<Body>) -> ResponseFuture {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Box::new(future::ok(
            Response::builder()
                .status(302)
                .header("Location", "/cargo_docserver")
                .body(Body::empty())
                .unwrap(),
        )),
        (&Method::GET, path) => {
            eprintln!("{}, not found", path);
            Box::new(
                tokio_fs::file::File::open("target/doc/cargo_docserver/index.html")
                    .and_then(|file| {
                        let buf: Vec<u8> = Vec::new();
                        tokio_io::io::read_to_end(file, buf)
                            .and_then(|item| Ok(Response::new(item.1.into())))
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
