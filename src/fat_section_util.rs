use crate::bios_parameter_block::{BYTES_PER_ENTRY, BYTES_PER_SECTOR, RESERVED_SECTORS};

pub fn get_fat_entry(bytes: &Vec<u8>, entry_num: usize) -> usize {
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

pub fn write_to_fat(mut bytes: Vec<u8>, entry_num: usize, last_entry_num: usize) -> Vec<u8> {
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
