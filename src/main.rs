use bootsector::edit_bootsector;
use directories::{change_directory, list_directory, make_directory};
use edit_file::editfile;
use new_file::newfile;
use read_file::read_file;
use root_dir_util::list_root_directory;
use shell_images::{close_image, create_new_image, open_image};
use shell_state::ShellState;
use std::io;
use std::io::*;

mod bios_parameter_block;
mod bootsector;
mod directories;
mod edit_file;
mod fat_section_util;
mod new_file;
mod read_file;
mod root_dir_util;
mod shell_images;
mod shell_parsing;
mod shell_state;

fn main() {
    // Stores the state of the shell (cwd, image bytes, etc)
    let mut shell_state = ShellState::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Couldn't read user input!");
        let args: Vec<&str> = input.trim().split(' ').collect();

        // Each arg has its own function
        shell_state = match args[0] {
            "open" => open_image(shell_state, args),
            "close" => close_image(shell_state, args),
            "new" => create_new_image(shell_state, args),
            "editboot" => edit_bootsector(shell_state, args),
            "newfile" => newfile(shell_state, args),
            "editfile" => editfile(shell_state, args),
            "ls" => {
                if shell_state.is_root() {
                    list_root_directory(shell_state, args)
                } else {
                    list_directory(shell_state, args)
                }
            }
            "cd" => change_directory(shell_state, args),
            "mkdir" => make_directory(shell_state, args),
            "save" => {
                println!("Saving file...");
                shell_state = shell_state.save_file();
                println!("File saved!");
                shell_state
            }
            "exit" => {
                break;
            }
            _ => {
                println!("Invalid function.");
                shell_state
            }
        };
    }
    println!("Finished!");
}
