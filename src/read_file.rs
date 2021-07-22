use crate::bios_parameter_block::BYTES_PER_SECTOR;
use crate::fat_section_util::get_fat_entry;
use crate::new_file::get_cluster_from_entry;
use crate::shell_state::ShellState;

pub fn read_file(shell_state: &ShellState, fat_entry: usize) -> Vec<u8> {
    let mut current_fat_entry = fat_entry;
    let file = vec![];

    loop {
        let mut cluster = get_cluster(&shell_state.bytes, current_fat_entry);
        file.append(&mut cluster);
        current_fat_entry = get_fat_entry(&shell_state.bytes, current_fat_entry);
        if current_fat_entry == 0xFFF {
            break;
        }
    }

    file
}

pub fn save_file_to_os(shell_state: ShellState, args: Vec<&str>) -> ShellState {
    let filename = get_arg(&args, 1).expect("Missing filename!");


    let file = read_file(&shell_state, )

    shell_state
}

fn get_cluster(bytes: &Vec<u8>, fat_entry: usize) -> Vec<u8> {
    let cluster_byte = get_cluster_from_entry(fat_entry);
    bytes[cluster_byte..cluster_byte + BYTES_PER_SECTOR].to_vec()
}
