use failure::{format_err, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::ops::Add;
use std::path::{Path, PathBuf};

#[deny(missing_docs)]
#[derive(Debug)]
/// Stores key-value pairs in memory.
pub struct KvStore {
    store: HashMap<String, String>,
    filepath: String,
    entries: i32,
}

pub type Result<T> = std::result::Result<T, Error>;

const FILE_NAME: &str = "log.txt";

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    key: String,
    value: String,
    kind: String,
}

impl KvStore {
    /// Creates an empty new key-value store using a hashmap data
    /// structure.
    pub fn new(filepath: String) -> KvStore {
        KvStore {
            store: HashMap::new(),
            filepath,
            entries: 0,
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cloned_key = String::from(&key);
        let cloned_value = String::from(&value);

        // It then serializes that command to a String
        let cmd = Command {
            key,
            value,
            kind: String::from("set"),
        };

        let cmd_string = serde_json::to_string(&cmd)?;

        // It then appends the serialized command to a file containing the log
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(String::from(&self.filepath))
            .unwrap();

        // If it fails, it exits by printing the error and returning a non-zero error code
        if let Err(e) = writeln!(file, "{}", cmd_string) {
            eprintln!("Couldn't write to file: {}", e);
            return Err(format_err!("Couldn't write to file: {}", e));
        }

        self.store.insert(cloned_key, cloned_value);

        self.entries += 1;

        self.compact();

        // If that succeeds, it exits silently with error code 0
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // kvs reads the entire log, one command at a time, recording the affected key and file offset of the command to an in-memory key -> log pointer map
        match KvStore::open(Path::new(&self.filepath)) {
            Ok(kvs) => self.store = kvs.store,
            Err(e) => {
                return Err(format_err!(
                    "Couldn't open file to insert key value pars: {}",
                    e
                ))
            }
        }

        // It then checks the map for the log pointer
        // If it succeeds
        // It deserializes the command to get the last recorded value of the key
        // It prints the value to stdout and exits with exit code 0
        return match self.store.get(&key) {
            Some(val) => Result::Ok(Some(String::from(val))),
            // If it fails, it prints "Key not found", and exits with exit code 0
            None => Result::Ok(None),
        };
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        // Same as the "get" command, kvs reads the entire log to build the in-memory index
        // It then checks the map if the given key exists
        // If the key does not exist, it prints "Key not found", and exits with a non-zero error code
        match self.get(key.clone()) {
            // Todo not sure if the clone is right
            Ok(option) => match option {
                Some(_) => {}
                None => {
                    return Err(format_err!("Key not found"));
                }
            },
            Err(e) => {
                return Err(e);
            }
        }

        let cloned_key = String::from(&key);

        // If it succeeds
        // It creates a value representing the "rm" command, containing its key
        let cmd = Command {
            key,
            value: String::new(),
            kind: String::from("rm"),
        };

        let cmd_string = serde_json::to_string(&cmd)?;

        // It then appends the serialized command to the log
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(String::from(&self.filepath))
            .unwrap();

        // If it fails, it exits by printing the error and returning a non-zero error code
        if let Err(e) = writeln!(file, "{}", cmd_string) {
            eprintln!("Couldn't write to file: {}", e);
            return Err(format_err!("Couldn't write to file: {}", e));
        }

        self.store.remove(&cloned_key);

        self.entries += 1;

        self.compact();

        // If that succeeds, it exits silently with error code 0
        return Ok(());
    }

    fn compact(&mut self) {
        if self.entries % 50 != 0 {
            return;
        }

        let data = self.to_serialized();

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.filepath)
            .expect("cannot open to file");

        f.write_all(data.as_ref()).expect("cannot write to file");

        f.flush().expect("cannot flush file buffer");
    }

    fn to_serialized(&self) -> String {
        let mut result = String::new();

        for (key, value) in &self.store {
            let cloned_key = String::from(key);
            let cloned_value = String::from(value);

            let cmd = Command {
                key: cloned_key,
                value: cloned_value,
                kind: String::from("set"),
            };

            let cmd_string = serde_json::to_string(&cmd).expect("cannot convert command to json");

            result = result.add(&cmd_string);
            result = result.add("\n");
        }

        result
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut into_path = path.into();

        if into_path.is_dir() {
            into_path.push(FILE_NAME);
        }

        let filepath = into_path.into_os_string().into_string().unwrap();

        let mut kvs = KvStore::new(filepath);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(String::from(&kvs.filepath))
            .unwrap();

        for line in io::BufReader::new(file).lines() {
            if let Ok(ip) = line {
                // Todo: It should ideally not deserialize every line, only those of the matching key.
                // Will make improvement.
                let cmd: Command = serde_json::from_str(ip.trim()).unwrap();

                if cmd.kind == "set" {
                    kvs.store.insert(cmd.key, cmd.value);
                } else if cmd.kind == "rm" {
                    kvs.store.remove(&cmd.key);
                }
            }
        }

        kvs.compact();

        Ok(kvs)
    }
}
