use crate::bios_parameter_block::{
    BYTES_PER_DIRECTORY_ENTRY, BYTES_PER_SECTOR, NUMBER_FATS, RESERVED_SECTORS, ROOT_ENTRIES,
    SECTORS_PER_FAT,
};
use crate::shell_state::ShellState;

pub fn append_to_root_dir(
    mut bytes: Vec<u8>,
    newfilename: String,
    first_entry: usize,
    is_subdir: bool,
) -> Vec<u8> {
    let root_start = (RESERVED_SECTORS + NUMBER_FATS * SECTORS_PER_FAT) * BYTES_PER_SECTOR;
    let root_entry = get_first_free_root_entry(&bytes);
    println!("First free root entry: {}", root_entry);

    let entry_start = root_start + BYTES_PER_DIRECTORY_ENTRY * root_entry;

    // Make sure filename is 11 letters long
    let mut truncated_filename = newfilename.clone();
    truncated_filename.truncate(11);
    let truncated_filename = format!("{:11}", truncated_filename).into_bytes();

    // I would make this a constant but it won't compile
    let mut entry_to_add = vec![0; 32];

    // Bytes 0-10: Filename and extension
    entry_to_add.splice(0..11, truncated_filename);

    // Byte 11: File attributes
    if is_subdir {
        entry_to_add[11] = 0b00001000;
    } else {
        entry_to_add[11] = 0b00000000;
    }

    // Bytes 26-27: First cluster
    entry_to_add[26] = (first_entry >> 8) as u8;
    entry_to_add[27] = first_entry as u8;

    // Bytes 28-32: File size
    // TODO

    bytes.splice(
        entry_start..entry_start + BYTES_PER_DIRECTORY_ENTRY,
        entry_to_add,
    );
    bytes
}

/// Returns 32 bit entry
pub fn read_root_entry(bytes: &Vec<u8>, root_entry: usize) -> Vec<u8> {
    let root_start = (RESERVED_SECTORS + NUMBER_FATS * SECTORS_PER_FAT) * BYTES_PER_SECTOR;
    let entry_start = root_start + BYTES_PER_DIRECTORY_ENTRY * root_entry;

    bytes[entry_start..entry_start + BYTES_PER_DIRECTORY_ENTRY].to_vec()
}

pub fn list_root_directory(shell_state: ShellState, _args: Vec<&str>) -> ShellState {
    println!("Listing files in the root directory:");
    println!("-----------------------");

    for root_entry_index in 0..ROOT_ENTRIES {
        let root_entry = read_root_entry(&shell_state.bytes, root_entry_index);
        if root_entry[0] != 0 {
            let filename = String::from_utf8(root_entry[0..12].to_vec()).unwrap();
            println!("{}", filename);
        }
    }

    println!("-----------------------");
    shell_state
}
