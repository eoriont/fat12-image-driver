use crate::bios_parameter_block::{
    BYTES_PER_DIRECTORY_ENTRY, BYTES_PER_SECTOR, NUMBER_FATS, RESERVED_SECTORS, ROOT_ENTRIES,
    SECTORS_PER_FAT,
};
use crate::fat_section_util::write_to_fat;
use crate::new_file::get_next_free_cluster;
use crate::root_dir_util::{append_to_root_dir, read_root_entry};
use crate::shell_parsing::get_arg;
use crate::shell_state::ShellState;

pub fn list_directory(shell_state: ShellState, _args: Vec<&str>) -> ShellState {
    println!("Listing files in current directory:");
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

pub fn change_directory(shell_state: ShellState, _args: Vec<&str>) -> ShellState {
    println!("Changing directories!");

    shell_state
}

pub fn make_directory(mut shell_state: ShellState, args: Vec<&str>) -> ShellState {
    println!("Made directory!");

    let mut dirname = get_arg(&args, 1).expect("Missing directory name!").clone();
    dirname.truncate(11);
    let formatted_dirname = format!("{:11}", dirname);

    let next_free_cluster = get_next_free_cluster(&shell_state.bytes, 0);
    shell_state.bytes = write_to_fat(shell_state.bytes, 0xFFF, next_free_cluster);

    if shell_state.is_root() {
        shell_state.bytes = append_to_root_dir(
            shell_state.bytes,
            formatted_dirname,
            next_free_cluster,
            true,
        );
    } else {
        shell_state.bytes = append_to_dir(
            shell_state.bytes,
            formatted_dirname,
            next_free_cluster,
            true,
            shell_state.get_cwd(),
        )
    }

    shell_state
}

pub fn append_to_dir(
    mut bytes: Vec<u8>,
    newfilename: String,
    first_entry: usize,
    is_subdir: bool,
    current_dir_fat_entry: usize,
) -> Vec<u8> {
    let dir_start = (RESERVED_SECTORS + NUMBER_FATS * SECTORS_PER_FAT) * BYTES_PER_SECTOR
        + ROOT_ENTRIES * BYTES_PER_DIRECTORY_ENTRY;
    let dir_entry = get_first_free_directory_entry(&bytes, current_dir_fat_entry);
    println!("First free root entry: {}", dir_entry);

    let entry_start = dir_entry + BYTES_PER_DIRECTORY_ENTRY * dir_entry;

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

pub fn get_first_free_directory_entry(bytes: &Vec<u8>, dir_start: usize) -> usize {
    for entry_index in 0.. {
        let dir_entry = read_directory_entry(&bytes, root_entry_index);
        // ! This is a hack, figure out how to test if entry is free
        if root_entry[0] == 0 {
            return root_entry_index;
        }
    }

    // This should crash
    0
}

/// Returns 32 bit entry
pub fn read_directory_entry(bytes: &Vec<u8>, root_entry: usize) -> Vec<u8> {
    let root_start = (RESERVED_SECTORS + NUMBER_FATS * SECTORS_PER_FAT) * BYTES_PER_SECTOR;
    let entry_start = root_start + BYTES_PER_DIRECTORY_ENTRY * root_entry;

    bytes[entry_start..entry_start + BYTES_PER_DIRECTORY_ENTRY].to_vec()
}
