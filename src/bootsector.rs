use crate::shell_parsing::get_arg;
use crate::shell_state::ShellState;

pub fn edit_bootsector(mut shell_state: ShellState, args: Vec<&str>) -> ShellState {
  // Get cmdline args
  let bootloader_filename = get_arg(&args, 1).expect("No bootloader filename provided!");

  // std::fs::read returns a vector of u8's
  let boot_bytes = std::fs::read(bootloader_filename).expect("Can't read the bootloader file!");

  // Make sure that the boot sector is only 512 bytes
  println!("Boot bytes len: {}", boot_bytes.len());
  assert_eq!(boot_bytes.len(), 512);

  // Replace first 512 bytes with boot sector
  shell_state.bytes.splice(..512, boot_bytes);

  println!("Attached bootsector!");

  shell_state
}
