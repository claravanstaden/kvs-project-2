use std::env;
use clap::{App, Arg};
use kvs::{KvStore, Result};

fn main() -> Result<()> {
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
                .arg(Arg::new("key")),
        )
        .get_matches();

    let mut kvs = KvStore::open(env::current_dir().unwrap())?;

    if let Some(matches) = matches.subcommand_matches("set") {
        let key = matches
            .value_of("key")
            .expect("unable to read key parameter for command set");

        let val = matches
            .value_of("value")
            .expect("unable to read value parameter for command set");

        kvs.set(String::from(key), String::from(val))
    } else if let Some(matches) = matches.subcommand_matches("get") {
        let key = matches
            .value_of("key")
            .expect("unable to read key parameter for command set");

        match kvs.get(String::from(key)) {
            Ok(value) => match value {
                Some(val) => println!("{}", val),
                None => println!("Key not found"),
            },
            Err(e) => println!("{}", e),
        }

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("rm") {
        let key = matches
            .value_of("key")
            .expect("unable to read key parameter for command set");

        match kvs.remove(String::from(key)) {
            Ok(_) => Ok(()),
            Err(_) => {
                println!("Key not found");
                std::process::exit(1);
            }
        }
    } else {
        panic!("invalid command")
    }
}
