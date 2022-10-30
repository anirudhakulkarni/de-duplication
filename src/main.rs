// create an API to save and load file with deduplication
// and compression

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::path::Path;
use std::fs;
// define constant
const CHUNK_SIZE: usize = 1024 * 1024;

//get hash of an vector (input type is Vec<u8>)
fn get_hash(input: &Vec<u8>) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();
    hash
}
fn get_last_chunk_index() -> u64 {
    // if state.txt does not exist create it
    let path = Path::new("state.txt");
    if !path.exists() {
        let mut file = File::create("state.txt").unwrap();
        file.write_all(b"0").unwrap();
        return 0;
    }
    // read state.txt
    let mut file = File::open("state.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let last_chunk_index = contents.parse::<u64>().unwrap();
    last_chunk_index + 1
    // File::open("state.txt").unwrap().read_to_string(&mut String::new()).unwrap()
}


fn checkifexist(file_path: &str)
// return hashmap of string to vector of strings
-> HashMap<String, Vec<String>> {
    let path = Path::new(file_path);
    if !path.exists() {
        let mut file = File::create(path).unwrap();
    }
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for line in contents.lines() {
        // line is comma seperated with first value as key and rest as values
        let mut line_iter = line.split(",");
        let key = line_iter.next().unwrap();
        let mut values: Vec<String> = line_iter.map(|x| x.to_string()).collect();
        // pop last value as it is empty
        if values[values.len() - 1] == "" {
            values.pop();
        }
        map.insert(key.to_string(), values);

    }

    map
}

fn setup(){
    // if state.txt does not exist create it and set it to 0
    let path = Path::new("state.txt");
    if !path.exists() {
        let mut file = File::create("state.txt").unwrap();
        file.write_all(b"0").unwrap();
    }
    // if map1.txt does not exist create it
    let path = Path::new("map1.txt");
    if !path.exists() {
        let mut file = File::create(path).unwrap();
    }
    // if map2.txt does not exist create it
    let path = Path::new("map2.txt");
    if !path.exists() {
        let mut file = File::create(path).unwrap();
    }
    // if chunks directory does not exist create it
    let path = Path::new("chunks");
    if !path.exists() {
        std::fs::create_dir(path).unwrap();
    }
    
}
fn clear(){
    fs::remove_file("map1.txt");
    fs::remove_file("map2.txt");
    fs::remove_file("state.txt");
    fs::remove_dir_all("chunks");
    
}

fn get_hashes
// return vector of tupe {chunk, hash} from vector of bytes
(input: &Vec<u8>) -> 
Vec<(Vec<u8>, u64)> {
    let mut hashes: Vec<(Vec<u8>, u64)> = Vec::new();
    let mut start = 0;
    let mut end = CHUNK_SIZE;
    while start < input.len() {
        if end > input.len() {
            end = input.len();
        }
        let chunk = input[start..end].to_vec();
        let hash = get_hash(&chunk);
        hashes.push((chunk, hash));
        start = end;
        end += CHUNK_SIZE;
    }
    
    hashes
}
pub fn save_file(path: &str) {
    setup();
    // let mut file = File::open(path).unwrap();
    // read file from path
    let mut file = File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    // get hashes of file
    let hashes = get_hashes(&buffer);

    let mut hashtochunk = checkifexist("map1.txt");
    let mut filetohashes = checkifexist("map2.txt");

    // append hashes to csv file with as filename, hasharray
    // if file exists append to it
    // else create new file
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("map2.txt")
        .unwrap();
    file.write_all(path.as_bytes()).unwrap();
    file.write_all(b",").unwrap();
    for hash in &hashes {
        file.write_all(hash.1.to_string().as_bytes()).unwrap();
        // file.write_all(hash.to_string().as_bytes()).unwrap();
        file.write_all(b",").unwrap();
    }
    file.write_all(b"\n").unwrap();




    //save chunks to disk
    let mut i = 0;
    // recover last chunk index
    i = get_last_chunk_index();

    // save only chunks which are not in disk

    for chunk_hash in &hashes{
        if !hashtochunk.contains_key(&chunk_hash.1.to_string()) {
            let mut file = File::create(format!("chunks/{}.chunk", i)).unwrap();
            file.write_all(&chunk_hash.0).unwrap();
            hashtochunk.insert(chunk_hash.1.to_string(), vec![format!("{}.chunk", i)]);
            // write to map1.txt
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open("map1.txt")
                .unwrap();
            file.write_all(chunk_hash.1.to_string().as_bytes()).unwrap();
            file.write_all(b",").unwrap();
            file.write_all(format!("{}.chunk", i).as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
            i += 1;
        }
    }

    // save last chunk index
    let mut file = File::create("state.txt").unwrap();
    file.write_all(i.to_string().as_bytes()).unwrap();

}

pub fn load_file(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut hashtochunk= checkifexist("map1.txt");
    let mut filetohashes = checkifexist("map2.txt");
    // println!("{:?}", hashtochunk);
    // println!("{:?}", filetohashes);
    
    let hashes = filetohashes.get(path.to_str().unwrap()).unwrap();
    // println!("{:?}", hashes);
    let mut chunks= Vec::new();
    for hash in hashes {
        // for hash get location of chunk
        // println!("{:?}", hash);      
        // println!("{:?}",hashtochunk.get(hash).unwrap()[0]);

        let mut file = File::open(format!("chunks/{}", hashtochunk.get(hash).unwrap()[0] )).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        chunks.push(buffer);
    }
    // return chunks
    let mut buffer = Vec::new();
    for chunk in chunks {
        buffer.extend(chunk);
    }
    // save file as file.txt
    // let mut file = File::create("file.jpeg").unwrap();
    // file.write_all(&buffer).unwrap();
    Ok(buffer)
}

fn main() {
    println!("Hello, world!");
    clear();
    save_file("memory.txt");
    save_file("mem.txt");
    let mem = load_file(Path::new("mem.txt")).unwrap();
    // save mem
    let mut file = File::create("file.jpeg").unwrap();
    file.write_all(&mem).unwrap();
    // load_file(Path::new("index.jpeg")).unwrap();
}
