use std::fs::{File, OpenOptions};
use std::io::prelude::*;

fn main() {
    let operation = std::env::args().nth(1).expect("No operation provided!");

    match operation.as_str() {
        "new" => create_new_image(),
        "editboot" => edit_bootloader(),
        _ => println!("Bruhhh"),
    }
    println!("Ran!");
}

fn create_new_image() {
    let filename = std::env::args().nth(2).expect("No filename provided!");
    let size = std::env::args().nth(3).expect("No size provided!");
    let size_in_mb: usize = size.parse().expect("Size isn't an integer!");

    let file = File::create(filename).expect("Can't create file!");
    file.set_len(((2 as usize).pow(20) * size_in_mb) as u64)
        .expect("Couldn't set file length");

    println!("Created file!")
}

fn edit_bootloader() {
    let filename = std::env::args().nth(2).expect("No filename provided!");
    let bootloader = std::env::args().nth(3).expect("No bootloader provided!");

    let mut bytes = std::fs::read(filename.clone()).expect("Can't read the image file!");

    let boot_bytes = std::fs::read(bootloader).expect("Can't read the bootloader file!");

    println!("Boot bytes len: {}", boot_bytes.len());
    assert_eq!(boot_bytes.len(), 512);

    bytes.splice(..512, boot_bytes);

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(filename)
        .expect("Can't open the image file!");
    file.write(&bytes).expect("Can't write data to file!");

    println!("Attached bootsector!")
}
