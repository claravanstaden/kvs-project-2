use clap::{App, Arg};
use kvs::KvStore;

fn main() {
    let matches = App::new("kvs")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Clara van Staden. <clara@snowfork.com>")
        .about("Stores key values in memory")
        .subcommand(App::new("set")
            .arg(Arg::new("key")
                .help("print debug information verbosely"))
            .arg(Arg::new("value")
                .help("print debug information verbosely")))
        .subcommand(App::new("get")
            .about("controls testing features")
            .arg(Arg::new("key")
                .help("print debug information verbosely")))
        .subcommand(App::new("rm")
            .about("controls testing features")
            .arg(Arg::new("rm")
                .help("print debug information verbosely")))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("set") {
        unimplemented!("set unimplemented")
    }
    else if let Some(matches) = matches.subcommand_matches("get") {
        unimplemented!("get unimplemented")
    }
    else if let Some(matches) = matches.subcommand_matches("rm") {
        unimplemented!("rm unimplemented")
    } else {
        panic!("invalid command")
    }
}