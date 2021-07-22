use crate::bios_parameter_block::{
    BITS_PER_FAT_ENTRY, BYTES_PER_DIRECTORY_ENTRY, BYTES_PER_SECTOR, NUMBER_FATS, RESERVED_SECTORS,
    ROOT_ENTRIES, SECTORS_PER_FAT,
};
use crate::fat_section_util::{get_fat_entry, write_to_fat};
use crate::root_dir_util::append_to_root_dir;
use crate::shell_parsing::get_arg;
use crate::shell_state::ShellState;

pub fn newfile(mut shell_state: ShellState, args: Vec<&str>) -> ShellState {
    // newfile image.bin testfile.txt
    // Get cmdline args
    let newfile: String = get_arg(&args, 1).expect("No new file provided!");

    let newfile_bytes = std::fs::read(newfile.clone()).expect("Can't read the image file!");

    let filename_extension = get_arg(&args, 2).expect("No new file name provided!");

    // newfile_bytes.len (ceildiv) BYTES_PER_SECTOR
    let newfile_sectors = (newfile_bytes.len() + BYTES_PER_SECTOR - 1) / BYTES_PER_SECTOR;

    let mut last_fat_entry: usize = 0;
    let mut sectors_stored = 0;
    let mut first_entry: usize = 0;
    // Is there more data left?
    while sectors_stored < newfile_sectors {
        //  Yes: get next free cluster
        let next_free_cluster = get_next_free_cluster(&shell_state.bytes, last_fat_entry);

        if sectors_stored == 0 {
            first_entry = next_free_cluster;
        }

        if next_free_cluster == 0 {
            // You're screwed (no free space)
            println!("You're screwed!");
            break;
        } else {
            //  put data at that cluster
            let cluster_byte = get_cluster_from_entry(next_free_cluster);

            let sector = get_cluster_from_new_file(&newfile_bytes, sectors_stored);

            shell_state.bytes = write_cluster(shell_state.bytes, cluster_byte, sector);

            // set last FAT entry to new cluster index
            if last_fat_entry != 0 && last_fat_entry != next_free_cluster {
                shell_state.bytes =
                    write_to_fat(shell_state.bytes, next_free_cluster, last_fat_entry);
            }

            // update last entry and keep looping
            last_fat_entry = next_free_cluster;
            sectors_stored += 1;
        }
    }
    //  No: set last FAT entry to EOF
    shell_state.bytes = write_to_fat(shell_state.bytes, 0xFFF, last_fat_entry);

    // Write to the root directory
    shell_state.bytes =
        append_to_root_dir(shell_state.bytes, filename_extension, first_entry, false);

    println!("Wrote new file to FAT12 Image!");

    shell_state
}

/// Gets a specific cluster from the new file and pads with 0s
fn get_cluster_from_new_file(new_file: &Vec<u8>, cluster_num: usize) -> Vec<u8> {
    let cluster_byte = cluster_num * BYTES_PER_SECTOR;
    let cluster_byte_end = cluster_byte + BYTES_PER_SECTOR;

    if new_file.len() < cluster_byte {
        // If cluster byte exceeds file length
        return vec![0; BYTES_PER_SECTOR];
    } else if new_file.len() < cluster_byte_end {
        // If cluster byte is within sector-aligned file
        let mut zeros = vec![0; cluster_byte_end - new_file.len()];
        let mut custom = new_file[cluster_byte..].to_vec();
        custom.append(&mut zeros);
        return custom;
    } else {
        // If cluster byte is within file
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

/// Returns first byte of the cluster
pub fn get_cluster_from_entry(entry: usize) -> usize {
    (RESERVED_SECTORS + (SECTORS_PER_FAT * NUMBER_FATS) + entry) * BYTES_PER_SECTOR
        + ROOT_ENTRIES * BYTES_PER_DIRECTORY_ENTRY
}

pub fn get_next_free_cluster(bytes: &Vec<u8>, current_entry: usize) -> usize {
    // Look through the FAT for first 0 entry
    let fat_start = RESERVED_SECTORS * BYTES_PER_SECTOR;
    let fat_end = fat_start + SECTORS_PER_FAT * BYTES_PER_SECTOR;

    let num_entries = (fat_end - fat_start) * 8 / BITS_PER_FAT_ENTRY;

    for entry_index in 0..num_entries {
        if get_fat_entry(&bytes, entry_index) == 0
            && entry_index != 0
            && entry_index != current_entry
        {
            // We found the next free cluster!
            return entry_index;
        }
    }
    // We should implement crashing because no free entry was found
    0
}
