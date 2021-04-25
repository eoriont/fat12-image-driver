use crate::bios_parameter_block::{
    BYTES_PER_ROOT_ENTRY, BYTES_PER_SECTOR, NUMBER_FATS, RESERVED_SECTORS, ROOT_ENTRIES,
    SECTORS_PER_FAT,
};

pub fn append_to_root_dir(mut bytes: Vec<u8>, newfilename: String, first_entry: usize) -> Vec<u8> {
    let root_start = (RESERVED_SECTORS + NUMBER_FATS * SECTORS_PER_FAT) * BYTES_PER_SECTOR;
    let root_entry = get_first_free_root_entry(&bytes);
    println!("First free root entry: {}", root_entry);

    let entry_start = root_start + BYTES_PER_ROOT_ENTRY * root_entry;

    // Make sure filename is 11 letters long
    let mut truncated_filename = newfilename.clone();
    truncated_filename.truncate(11);
    let truncated_filename = format!("{:11}", truncated_filename).into_bytes();

    // I would make this a constant but it won't compile
    let mut entry_to_add = vec![0; 32];

    // Bytes 0-11: Filename and extension
    entry_to_add.splice(0..12, truncated_filename);

    // Bytes 26-27: First cluster
    entry_to_add[26] = (first_entry >> 8) as u8;
    entry_to_add[27] = first_entry as u8;

    // Bytes 28-32: File size
    // TODO

    bytes.splice(
        entry_start..entry_start + BYTES_PER_ROOT_ENTRY,
        entry_to_add,
    );
    bytes
}

pub fn get_first_free_root_entry(bytes: &Vec<u8>) -> usize {
    for root_entry_index in 0..ROOT_ENTRIES {
        let root_entry = read_root_entry(&bytes, root_entry_index);
        // ! This is a hack, figure out how to test if entry is free
        if root_entry[0] == 0 {
            return root_entry_index;
        }
    }

    // This should crash
    0
}

/// Returns 32 bit entry
pub fn read_root_entry(bytes: &Vec<u8>, root_entry: usize) -> Vec<u8> {
    let root_start = (RESERVED_SECTORS + NUMBER_FATS * SECTORS_PER_FAT) * BYTES_PER_SECTOR;
    let entry_start = root_start + BYTES_PER_ROOT_ENTRY * root_entry;

    bytes[entry_start..entry_start + BYTES_PER_ROOT_ENTRY].to_vec()
}
