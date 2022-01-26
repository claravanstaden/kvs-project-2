#Key Value Store

Stores key value pairs with a log backing. The program compacts the key store after 50 entries, meaning that duplicate keys will be cleaned up, for instance:  
```
set key1=value1  
set key1=value2  
```
After compact, the key store would be:
```
set key1=value2  
```

Similarly, remove statements will remove the key from the log.
```
set key1=value1  
set key2=value2  
rm key1
```
After compact, the key store would be:
```
set key2=value2  
```

The key values are stored in JSON format. `gjson` is used to lookup commands (`set`/`rm`) and keys instead of deserializing each command. The command is only deserialized when the value needs to be returned.
## Project Background

Implements [project 2 of the Pingcap](https://github.com/pingcap/talent-plan/blob/master/courses/rust/projects/project-2/README.md) talent plan Rust exercises.

## How to Run

### Add a key value
Add a new key value pair.
```
$ cargo run -- set <key> <value>
```

e.g.
```
$ cargo run -- set key1 value1
```

### Get Value at Key
Prints the value stored in a given key.
```
$ cargo run -- get <key>
```
e.g.  
Get value at key that exists.
```
$ cargo run -- get key1
value1
```
Get value at key that doesn't exist.
```
$ cargo run -- get key2
Key not found
```
### Remove key
```
$ cargo run -- rm key1
```
e.g.  
Remove value at key that exists.
```
$ cargo run -- rm key1
```
Remove value at key that doesn't exist.
```
$ cargo run -- rm key2
Key not found
```


## Tests
```
$ cargo run tests
```

```
running 17 tests
test cli_invalid_subcommand ... ok
test cli_rm_non_existent_key ... ok
test cli_get_non_existent_key ... ok
test cli_version ... ok
test cli_set ... ok
test cli_get_stored ... ok
test get_non_existent_value ... ok
test get_stored_value ... ok
test remove_key ... ok
test cli_rm_stored ... ok
test overwrite_value ... ok
test remove_non_existent_key ... ok
test cli_no_args ... ok
test cli_invalid_get ... ok
test cli_invalid_rm ... ok
test cli_invalid_set ... ok
test compaction ... ok
```