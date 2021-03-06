use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::ops::Add;
use std::path::{PathBuf};
use crate::CustomError::{InvalidPathError, KeyNotFound, WriteFileError};

#[deny(missing_docs)]
#[derive(Debug)]
/// Stores key-value pairs in memory.
pub struct KvStore {
    store: HashMap<String, i32>,
    filepath: String,
    entries: i32,
}

pub enum CustomError {
    InvalidPathError,
    WriteFileError,
    KeyNotFound,
}

impl Debug for CustomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {:?}", self)
    }
}

impl Display for CustomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {:?}", self)
    }
}

impl Error for CustomError {}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

const FILE_NAME: &str = "log.txt";
const COMPACT_FREQUENCY: i32 = 50;

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

    /// Sets a key with a value in the store.
    /// # Examples
    /// ```
    /// use kvs::KvStore;
    /// let mut kvs = KvStore::new(String::from("."));
    ///
    /// let key = String::from("foo");
    /// let value = String::from("bar");
    ///
    /// kvs.set(key, value);
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cloned_key = String::from(&key);

        let cmd = Command {
            key,
            value,
            kind: String::from("set"),
        };

        let cmd_string = serde_json::to_string(&cmd)?;

        let file = BufReader::new(File::open(&self.filepath)?);
        let mut line_count = 0;

        for _ in file.lines() {
            line_count = line_count + 1;
        }

        let mut file2 = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(String::from(&self.filepath))?;

        if let Err(e) = writeln!(file2, "{}", cmd_string) {
            return Err(Box::new(e));
        }

        self.store.insert(cloned_key, line_count + 1);

        self.entries += 1;

        self.compact()?;

        Ok(())
    }

    /// Gets a value in the store using the key.
    /// # Examples
    /// ```
    /// use kvs::KvStore;
    /// let mut kvs = KvStore::new(String::from("."));
    ///
    /// let key = String::from("foo");
    /// let value = String::from("bar");
    ///
    /// kvs.set(key, value);
    ///
    /// print!(kvs.get(String::from("foo")));
    /// ```
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let index = match self.store.get(&key) {
            Some(val) => val,
            None => return Result::Ok(None),
        };

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(String::from(&self.filepath))?;

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

        let cmd: Command = serde_json::from_str(cmd_string.trim())?;

        Result::Ok(Some(cmd.value))
    }

    /// Removes the value and key in the store using the key.
    /// # Examples
    /// ```
    /// use kvs::KvStore;
    /// let mut kvs = KvStore::new(String::from("."));
    ///
    /// let key = String::from("foo");
    /// let value = String::from("bar");
    ///
    /// kvs.set(key, value);
    ///
    /// kvs.rm(String::from("foo"));
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.get(key.clone()) {
            Ok(option) => match option {
                Some(_) => {}
                None => return Err(Box::new(KeyNotFound))
            },
            Err(e) => {
                return Err(e);
            }
        }

        let cloned_key = String::from(&key);

        let cmd = Command {
            key,
            value: String::new(),
            kind: String::from("rm"),
        };

        let cmd_string = serde_json::to_string(&cmd)?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(String::from(&self.filepath))?;

        if let Err(_) = writeln!(file, "{}", cmd_string) {
            return Err(Box::new(WriteFileError));
        }

        self.store.remove(&cloned_key);

        self.entries += 1;

        self.compact()?;

        return Ok(());
    }

    fn compact(&mut self) -> Result<()> {
        if self.entries % COMPACT_FREQUENCY != 0 {
            return Ok(());
        }

        let data = self.to_serialized();

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.filepath)?;

        f.write_all(data.as_ref()).expect("cannot write to file");

        f.flush().expect("cannot flush file buffer");

        self.load_into_memory()?;

        Ok(())
    }

    fn to_serialized(&self) -> String {
        let file = File::open(&self.filepath).expect("no such file");
        let buf = BufReader::new(file);
        let content: Vec<String> = buf.lines()
            .map(|l| l.expect("Could not parse line"))
            .collect();

        let mut result = String::new();

        let mut tmp_map: HashMap<String, String> = HashMap::new();

        for item in content {
            let cmd = gjson::get(&item, "kind");
            let key = gjson::get(&item, "key");

            if cmd.str() == "set" {
                tmp_map.insert(String::from(key.str()), item);
            } else if cmd.str() == "rm" {
                tmp_map.remove(&String::from(key.str()));
            }
        }

        for (_, value) in tmp_map {
            result = result.add(&value);
            result = result.add("\n");
        }

        result
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut into_path = path.into();

        if into_path.is_dir() {
            into_path.push(FILE_NAME);
        }

        let filepath = into_path
            .into_os_string()
            .into_string()
            .or(Err(Box::new(InvalidPathError)))?;

        let mut kvs = KvStore::new(filepath);

        kvs.load_into_memory()?;

        Ok(kvs)
    }

    fn load_into_memory(&mut self) -> Result<()> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(String::from(&self.filepath))?;

        let mut line_count = 0;

        self.entries = 0;

        self.store = HashMap::new();

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
        };

        Ok(())
    }
}
