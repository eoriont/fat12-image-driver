use shell_state::ShellState;
use std::fs::File;
use std::io;
use std::io::*;

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
            "new" => create_new_image(shell_state, args),
            "editboot" => edit_bootloader(shell_state, args),
            "newfile" => newfile(shell_state, args),
            "editfile" => editfile(shell_state, args),
            "lsroot" => list_root(shell_state, args),
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

fn open_image(shell_state: ShellState, args: Vec<&str>) -> ShellState {
    let image_filename = get_arg(&args, 1).expect("Couldn't open image file!");
    shell_state.open_file(image_filename)
}

fn create_new_image(shell_state: ShellState, args: Vec<&str>) -> ShellState {
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

fn edit_bootloader(mut shell_state: ShellState, args: Vec<&str>) -> ShellState {
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

fn newfile(mut shell_state: ShellState, args: Vec<&str>) -> ShellState {
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
    shell_state.bytes = append_to_root_dir(shell_state.bytes, filename_extension, first_entry);

    println!("Wrote new file to FAT12 Image!");

    shell_state
}

fn list_root(shell_state: ShellState, _args: Vec<&str>) -> ShellState {
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

fn append_to_root_dir(mut bytes: Vec<u8>, newfilename: String, first_entry: usize) -> Vec<u8> {
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

fn get_first_free_root_entry(bytes: &Vec<u8>) -> usize {
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
fn read_root_entry(bytes: &Vec<u8>, root_entry: usize) -> Vec<u8> {
    let root_start = (RESERVED_SECTORS + NUMBER_FATS * SECTORS_PER_FAT) * BYTES_PER_SECTOR;
    let entry_start = root_start + BYTES_PER_ROOT_ENTRY * root_entry;

    bytes[entry_start..entry_start + BYTES_PER_ROOT_ENTRY].to_vec()
}

fn write_to_fat(mut bytes: Vec<u8>, entry_num: usize, last_entry_num: usize) -> Vec<u8> {
    let fat_start = RESERVED_SECTORS * BYTES_PER_SECTOR;
    let entry_start = fat_start + (last_entry_num as f64 * BYTES_PER_ENTRY) as usize;
    let mut new_bytes = bytes[entry_start..entry_start + 2].to_vec();

    if last_entry_num % 2 == 0 {
        // If the entry is even
        // xxxx xxxx xxxx 0000
        new_bytes[0] = (entry_num >> 4) as u8;
        new_bytes[1] = (entry_num as u8) << 4 + (new_bytes[1] & 0b00001111);
    } else {
        // if the entry is odd
        // 0000 xxxx xxxx xxxx
        new_bytes[0] = (entry_num >> 8) as u8 + (new_bytes[0] & 0b11110000);
        new_bytes[1] = entry_num as u8;
    }

    bytes.splice(entry_start..entry_start + 2, new_bytes);
    bytes
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
fn get_cluster_from_entry(entry: usize) -> usize {
    (RESERVED_SECTORS + (SECTORS_PER_FAT * NUMBER_FATS) + entry) * BYTES_PER_SECTOR
        + ROOT_ENTRIES * BYTES_PER_ROOT_ENTRY
}

fn get_next_free_cluster(bytes: &Vec<u8>, current_entry: usize) -> usize {
    // Look through the FAT for first 0 entry
    let fat_start = RESERVED_SECTORS * BYTES_PER_SECTOR;
    let fat_end = fat_start + SECTORS_PER_FAT * BYTES_PER_SECTOR;

    let num_entries = (fat_end - fat_start) * 8 / BITS_PER_ENTRY;

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
        return (((untrimmed_bytes[0] & 0b00001111) as usize) << 8) + untrimmed_bytes[1] as usize;
    }
}

fn editfile(shell_state: ShellState, _args: Vec<&str>) -> ShellState {
    shell_state
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
const BYTES_PER_ROOT_ENTRY: usize = 32;

fn get_arg(args: &Vec<&str>, argnum: usize) -> Result<String> {
    if args.len() > argnum {
        return Ok(args[argnum].to_owned());
    } else {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Not enough arguments!",
        ));
    }
}
