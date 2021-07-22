pub const OEM: &str = "My OS   ";
pub const BYTES_PER_SECTOR: usize = 512;
pub const SECTORS_PER_CLUSTER: usize = 1;
pub const RESERVED_SECTORS: usize = 1;
pub const NUMBER_FATS: usize = 2;
pub const ROOT_ENTRIES: usize = 224;
pub const TOTAL_SECTORS: usize = 2880;
pub const MEDIA: usize = 0xf8;
pub const SECTORS_PER_FAT: usize = 9;
pub const SECTORS_PER_TRACK: usize = 18;
pub const HEADS_PER_CYLINDER: usize = 2;
pub const HIDDEN_SECTORS: usize = 0;
pub const TOTAL_SECTORS_BIG: usize = 0;
pub const DRIVE_NUMBER: usize = 0;
pub const UNUSED: usize = 0;
pub const EXT_BOOT_SIGNATURE: usize = 0x29;
pub const SERIAL_NUMBER: usize = 0xa0a1a2a3;
pub const VOLUME_LABEL: &str = "MOS FLOPPY ";
pub const FILE_SYSTEM: &str = "FAT12   ";
pub const BITS_PER_FAT_ENTRY: usize = 12;
pub const BYTES_PER_ENTRY: f64 = 3.0 / 2.0;
pub const BYTES_PER_DIRECTORY_ENTRY: usize = 32;
