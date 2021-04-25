use crate::bios_parameter_block::ROOT_ENTRIES;
use crate::root_dir_util::read_root_entry;
use crate::shell_state::ShellState;

pub fn list_directory(shell_state: ShellState, _args: Vec<&str>) -> ShellState {
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
