use failure::{format_err, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::ops::Add;
use std::path::{Path, PathBuf};

#[deny(missing_docs)]
#[derive(Debug)]
/// Stores key-value pairs in memory.
pub struct KvStore {
    store: HashMap<String, i32>,
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

        // It then serializes that command to a String
        let cmd = Command {
            key,
            value,
            kind: String::from("set"),
        };

        let cmd_string = serde_json::to_string(&cmd)?;

        let file = BufReader::new(File::open(&self.filepath).expect("Unable to open file"));
        let mut line_count = 0;

        for _ in file.lines() {
            line_count = line_count + 1;
        }

        let mut file2 = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(String::from(&self.filepath))
            .unwrap();

        // If it fails, it exits by printing the error and returning a non-zero error code
        if let Err(e) = writeln!(file2, "{}", cmd_string) {
            eprintln!("Couldn't write to file: {}", e);
            return Err(format_err!("Couldn't write to file: {}", e));
        }

        self.store.insert(cloned_key, line_count + 1);

        self.entries += 1;

        self.compact();

        // If that succeeds, it exits silently with error code 0
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // It then checks the map for the log pointer
        let index = match self.store.get(&key) {
            Some(val) => val,
            // If it fails, it prints "Key not found", and exits with exit code 0
            None => return Result::Ok(None),
        };

        // If it succeeds
        // It deserializes the command to get the last recorded value of the key
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(String::from(&self.filepath))
            .unwrap();

        let mut line_count = 0;

        let mut cmd_string = String::new();

        for line in io::BufReader::new(file).lines() {
            if let Ok(ip) = line {
                line_count = line_count + 1;

                if line_count == *index {
                    cmd_string = ip;
                    break;
                }
            }
        }

        if cmd_string == "" {
            return Result::Ok(None);
        }

        let cmd: Command = serde_json::from_str(cmd_string.trim()).unwrap();

        // It prints the value to stdout and exits with exit code 0
        Result::Ok(Some(cmd.value))
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        // Same as the "get" command, kvs reads the entire log to build the in-memory index
        // It then checks the map if the given key exists
        // If the key does not exist, it prints "Key not found", and exits with a non-zero error code
        match self.get(key.clone()) {
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

        self.load_into_memory();
    }

    fn to_serialized(&self) -> String {
        let file = File::open(&self.filepath).expect("no such file");
        let buf = BufReader::new(file);
        let content: Vec<String> = buf.lines()
            .map(|l| l.expect("Could not parse line"))
            .collect();

        let mut result = String::new();

        let mut swap_keys_values: HashMap<i32, String> = HashMap::new();

        for (key, value) in &self.store {
            swap_keys_values.insert(*value, String::from(key));
        }

        for n in 1..=swap_keys_values.len() {
            let index = n - 1;
            let cloned_value = content.get(index as usize).expect("cannot read value from file");

            result = result.add(&cloned_value);
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

        kvs.load_into_memory();

        Ok(kvs)
    }

    fn load_into_memory(&mut self) {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(String::from(&self.filepath))
            .unwrap();

        let mut line_count = 0;

        for line in io::BufReader::new(file).lines() {
            if let Ok(ip) = line {
                let cmd = gjson::get(&ip, "kind");
                let key = gjson::get(&ip, "key");

                line_count = line_count + 1;

                if cmd.str() == "set" {
                    self.store.insert(String::from(key.str()), line_count);
                } else if cmd.str() == "rm" {
                    self.store.remove(&String::from(key.str()));
                }

                self.entries = self.entries + 1;
            }
        }
    }
}
