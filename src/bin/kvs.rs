use clap::{App, Arg};
use kvs::KvStore;

fn main() {
    let matches = App::new("kvs")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Clara van Staden. <clara@snowfork.com>")
        .about("Stores key values in memory")
        .subcommand(
            App::new("set")
                .about("sets a new key-value pair")
                .arg(Arg::new("key"))
                .arg(Arg::new("value")),
        )
        .subcommand(
            App::new("get")
                .about("gets a value based on the key")
                .arg(Arg::new("key")),
        )
        .subcommand(
            App::new("rm")
                .about("remove a value from the key store")
                .arg(Arg::new("rm")),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("set") {
        unimplemented!("set unimplemented")
    } else if let Some(matches) = matches.subcommand_matches("get") {
        unimplemented!("get unimplemented")
    } else if let Some(matches) = matches.subcommand_matches("rm") {
        unimplemented!("rm unimplemented")
    } else {
        panic!("invalid command")
    }
}
