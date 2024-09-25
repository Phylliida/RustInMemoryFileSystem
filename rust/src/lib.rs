#![allow(unused_variables)]
#![allow(dead_code)]

use crate::filesystem::FS;

// TODO: STATUS_ON_STORAGE STUFF (local filestorage? can't in pure wasm)
// TODO: clone_me rename to clone once I'm done with porting (For Uint8Array)
// TODO: Is it ok to clone FSLockRegion vec?

// https://github.com/wasm-forge/ their file system seems good, doesn't support symlinks but otherwise
// it came out at the same time as this :/
mod filesystem;
mod marshall;
mod v9p;

#[no_mangle]
pub extern "C" fn main() {
    let mut fs = FS::new(None);
    let path = "bees.bepis";
    let beesptr = fs.create_text_file(path, fs.root_id, "applebeeeeeees");
    println!("got file {}", beesptr);
    if let Some(result_text) = fs.read_text_file(path) {
        println!("got text {}", result_text);
    }
    else {
        println!("got no text");
    }

    let directory = fs.create_directory("applebees wow", Some(fs.root_id));
    let beesptr2 = fs.create_text_file(path, directory, "applebeeeeeees2");
    println!("got file 2 {} with full path {}", beesptr2, fs.get_full_path(directory));
    if let Some(result_text) = fs.read_text_file("/applebees wow/bees.bepis") {
        println!("got text 2 {}", result_text);
    }
    else {
        println!("got no text 2");
    }
    
} 

