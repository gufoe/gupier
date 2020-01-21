extern crate kiss3d;
extern crate nalgebra as na;
extern crate serde;

mod world;
mod util;
mod server;
mod client;
mod com;

mod litio_host;
mod litio_client;
mod gameplay;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(short, long)]
    serve: Option<u16>,

    #[structopt(short, long)]
    connect: Option<String>,
}


fn main() {
    let opt = Opt::from_args();

    let gp_host = litio_host::Host::new();
    let gp_client = litio_client::Client::new();

    if opt.serve.is_some() {
        server::serve(opt.serve.unwrap(), gp_host);
    } else if opt.connect.is_some() {
        client::connect(opt.connect.unwrap(), gp_client);
    } else {
        std::thread::spawn(|| {
            server::serve(20016, gp_host);
        });
        client::connect("127.0.0.1:20016".to_string(), gp_client);
    }
}
