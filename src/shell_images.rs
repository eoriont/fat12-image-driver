use crate::shell_parsing::get_arg;
use crate::shell_state::ShellState;
use std::fs::File;

pub fn open_image(shell_state: ShellState, args: Vec<&str>) -> ShellState {
    let image_filename = get_arg(&args, 1).expect("Couldn't open image file!");
    shell_state.open_file(image_filename)
}

pub fn create_new_image(shell_state: ShellState, args: Vec<&str>) -> ShellState {
    // Get cmdline args
    let filename = get_arg(&args, 1).expect("No filename provided!");
    let size_str = get_arg(&args, 2).expect("No size provided!");

    // Parse size string
    let size_in_mb: usize = size_str.parse().expect("Size isn't an integer!");

    // Create the file
    let file = File::create(&filename).expect("Can't create file!");

    // Setting the length to longer than the file is just fills it with 0's
    file.set_len(((2 as usize).pow(20) * size_in_mb) as u64)
        .expect("Couldn't set file length");

    println!("Created file!");

    shell_state.open_file(filename)
}

pub fn close_image(_shell_state: ShellState, _args: Vec<&str>) -> ShellState {
    ShellState::new()
}
