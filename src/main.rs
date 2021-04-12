use std::fs::{File, OpenOptions};
use std::io::prelude::*;

fn main() {
    // First cmdline arg
    let operation = std::env::args().nth(1).expect("No operation provided!");

    // Each arg has its own function
    match operation.as_str() {
        "new" => create_new_image(),
        "editboot" => edit_bootloader(),
        "newfile" => newfile(),
        "editfile" => editfile(),
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

fn newfile() {
    // cargo run newfile image.bin boot.bin
    // Get cmdline args
    let image_file = std::env::args()
        .nth(2)
        .expect("No image filename provided!");
    let newfile: String = std::env::args().nth(3).expect("No new file provided!");

    let mut bytes = get_image_data(&image_file);
    let newfile_bytes = std::fs::read(newfile.clone()).expect("Can't read the image file!");

    // let filename_extension = newfile.as_bytes();

    // newfile_bytes.len (ceildiv) BYTES_PER_SECTOR
    let newfile_sectors = (newfile_bytes.len() + BYTES_PER_SECTOR - 1) / BYTES_PER_SECTOR;

    let mut last_fat_entry: usize = 0;
    let mut sectors_stored = 0;
    // Is there more data left?
    while sectors_stored < newfile_sectors {
        //  Yes: get next free cluster
        let next_free_cluster = get_next_free_cluster(&bytes);
        if next_free_cluster == 0 {
            // You're screwed (no free space)
            println!("You're screwed!");
            break;
        } else {
            //  put data at that cluster
            let cluster_byte = get_cluster_from_entry(next_free_cluster);

            let sector = get_cluster_from_new_file(&newfile_bytes, sectors_stored);

            bytes = write_cluster(bytes, cluster_byte, sector);

            // set last FAT entry to new cluster index
            if last_fat_entry != 0 {
                bytes = write_to_fat(bytes, next_free_cluster, last_fat_entry);
            }

            // update last entry and keep looping
            last_fat_entry = next_free_cluster;
            sectors_stored += 1;
        }
    }
    //  No: set last FAT entry to EOF
    bytes = write_to_fat(bytes, last_fat_entry, 0xFFF);

    // Opening a file with truncate will replace contents
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(image_file)
        .expect("Can't open the image file!");
    file.write(&bytes).expect("Can't write data to file!");

    println!("Wrote new file to FAT12 Image!");
}

fn write_to_fat(mut bytes: Vec<u8>, entry_num: usize, last_entry_num: usize) -> Vec<u8> {
    let fat_start = RESERVED_SECTORS * BYTES_PER_SECTOR;
    let entry_start = fat_start + (entry_num as f64 * BYTES_PER_ENTRY) as usize;
    let mut new_bytes = bytes[entry_start..entry_start + 2].to_vec();

    if entry_num % 2 == 0 {
        // If the entry is even
        new_bytes[0] = (last_entry_num >> 4) as u8;
        new_bytes[1] = (last_entry_num as u8) << 4 + (new_bytes[1] & 0b00001111);
    } else {
        // if the entry is odd
        new_bytes[0] = (last_entry_num >> 8) as u8 + (new_bytes[0] & 0b11110000);
        new_bytes[1] = last_entry_num as u8;
    }

    bytes.splice(entry_start..entry_start + 2, new_bytes);
    bytes
}

/// Gets a specific cluster from the new file and pads with 0s
fn get_cluster_from_new_file(new_file: &Vec<u8>, cluster_num: usize) -> Vec<u8> {
    let cluster_byte = cluster_num * BYTES_PER_SECTOR;
    let cluster_byte_end = cluster_byte + BYTES_PER_SECTOR;
    if new_file.len() < cluster_byte {
        return vec![0; BYTES_PER_SECTOR];
    } else if new_file.len() < cluster_byte_end {
        let mut zeros = vec![0; cluster_byte_end - new_file.len()];
        let mut custom = new_file[cluster_byte..].to_vec();
        custom.append(&mut zeros);
        return custom;
    } else {
        return new_file[cluster_byte..cluster_byte_end].to_vec();
    }
}

/// cluster_byte is the byte index of the start of the cluster
fn write_cluster(mut bytes: Vec<u8>, cluster_byte: usize, cluster_to_write: Vec<u8>) -> Vec<u8> {
    bytes.splice(
        cluster_byte..cluster_byte + BYTES_PER_SECTOR,
        cluster_to_write,
    );
    bytes
}

// Returns first byte of the cluster
fn get_cluster_from_entry(entry: usize) -> usize {
    (RESERVED_SECTORS + (SECTORS_PER_FAT * NUMBER_FATS) + entry) * BYTES_PER_SECTOR
        + (ROOT_ENTRIES as f64 * BYTES_PER_ENTRY) as usize
}

fn get_next_free_cluster(bytes: &Vec<u8>) -> usize {
    // Look through the FAT for first 0 entry
    let fat_start = RESERVED_SECTORS * BYTES_PER_SECTOR;
    let fat_end = fat_start + SECTORS_PER_FAT * BYTES_PER_SECTOR;

    let num_entries = (fat_end - fat_start) * 8 / BITS_PER_ENTRY;

    for entry_index in 0..num_entries {
        if get_fat_entry(&bytes, entry_index) == 0 && entry_index != 0 {
            // We found the next free cluster!
            return entry_index;
        }
    }
    // We should implement crashing because no free entry was found
    0
}

fn get_fat_entry(bytes: &Vec<u8>, entry_num: usize) -> usize {
    // Realistically should return a u12 but that doesn't exist
    let fat_start = RESERVED_SECTORS * BYTES_PER_SECTOR;

    let entry_start_byte = fat_start + (entry_num as f64 * BYTES_PER_ENTRY) as usize;
    let untrimmed_bytes = &bytes[entry_start_byte..entry_start_byte + 2];

    if entry_num % 2 == 0 {
        // If entry is even
        // xxxx xxxx xxxx 0000
        return ((untrimmed_bytes[0] as usize) << 4) + (untrimmed_bytes[1] >> 4) as usize;
    } else {
        // if entry is odd
        // 0000 xxxx xxxx xxxx
        return (((untrimmed_bytes[0] as usize) & 0b00001111) << 8) + untrimmed_bytes[1] as usize;
    }
}

fn editfile() {}

fn get_image_data(filename: &String) -> Vec<u8> {
    std::fs::read(filename).expect("Can't read the image file!")
}

// const OEM: &str = "My OS   ";
const BYTES_PER_SECTOR: usize = 512;
// const SECTORS_PER_CLUSTER: usize = 1;
const RESERVED_SECTORS: usize = 1;
const NUMBER_FATS: usize = 2;
const ROOT_ENTRIES: usize = 224;
// const TOTAL_SECTORS: usize = 2880;
// const MEDIA: usize = 0xf8;
const SECTORS_PER_FAT: usize = 9;
// const SECTORS_PER_TRACK: usize = 18;
// const HEADS_PER_CYLINDER: usize = 2;
// const HIDDEN_SECTORS: usize = 0;
// const TOTAL_SECTORS_BIG: usize = 0;
// const DRIVE_NUMBER: usize = 0;
// const UNUSED: usize = 0;
// const EXT_BOOT_SIGNATURE: usize = 0x29;
// const SERIAL_NUMBER: usize = 0xa0a1a2a3;
// const VOLUME_LABEL: &str = "MOS FLOPPY ";
// const FILE_SYSTEM: &str = "FAT12   ";
const BITS_PER_ENTRY: usize = 12;
const BYTES_PER_ENTRY: f64 = 3.0 / 2.0;
