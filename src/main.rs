use std::fs::{File, OpenOptions};
use std::io::prelude::*;

fn main() {
    // First cmdline arg
    let operation = std::env::args().nth(1).expect("No operation provided!");

    // Each arg has its own function
    match operation.as_str() {
        "new" => create_new_image(),
        "editboot" => edit_bootloader(),
        _ => println!("Bruhhh"),
    }
    println!("Finished!");
}

fn create_new_image() {
    // Get cmdline args
    let filename = std::env::args().nth(2).expect("No filename provided!");
    let size = std::env::args().nth(3).expect("No size provided!");
    let size_in_mb: usize = size.parse().expect("Size isn't an integer!");

    // Create the file
    let file = File::create(filename).expect("Can't create file!");

    // Setting the length to longer than the file is just fills it with 0's
    file.set_len(((2 as usize).pow(20) * size_in_mb) as u64)
        .expect("Couldn't set file length");

    println!("Created file!")
}

fn edit_bootloader() {
    // Get cmdline args
    let filename = std::env::args().nth(2).expect("No filename provided!");
    let bootloader = std::env::args().nth(3).expect("No bootloader provided!");

    // std::fs::read returns a vector of u8's
    let mut bytes = std::fs::read(filename.clone()).expect("Can't read the image file!");
    let boot_bytes = std::fs::read(bootloader).expect("Can't read the bootloader file!");

    // Make sure that the boot sector is only 512 bytes
    println!("Boot bytes len: {}", boot_bytes.len());
    assert_eq!(boot_bytes.len(), 512);

    // Replace first 512 bytes with boot sector
    bytes.splice(..512, boot_bytes);

    // Opening a file with truncate will replace contents
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(filename)
        .expect("Can't open the image file!");
    file.write(&bytes).expect("Can't write data to file!");

    println!("Attached bootsector!")
}
