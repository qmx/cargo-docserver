# cargo-docserver - exposes your crate docs in a built-in http server

This is just the result of me fooling around with what would be the minimal HTTP server for this. This is not security-vetted, you've been warned. Use only for development, if you're feeling brave

## usage:

`cargo docserver -p <port>`

Additionally you can compile the docs while the server is running by pressing ENTER. You can pass arguments to `cargo doc` with `-r "--some-arg"`. For example `cargo docserver -r "--no-deps -j 2"`


### sidenote

I've tried really hard to find a simple embeddable rust static file server, and found several, not necessarily fitting the bill here. The closest one was [static-server](https://github.com/DenisKolodin/static-server) but it loads everything in memory :(

if you happen to stumble into one, let me know! happy to ditch my hacky http server :P
