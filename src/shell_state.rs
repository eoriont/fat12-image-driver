use std::fs::OpenOptions;
use std::io::Write;

pub struct ShellState {
  image_filename: String,
  pub bytes: Vec<u8>,
  cwd_fat_entry: usize,
  is_image_file_open: bool,
  is_root: bool,
}

impl ShellState {
  pub fn new() -> Self {
    ShellState {
      image_filename: String::default(),
      bytes: vec![],
      cwd_fat_entry: 0,
      is_image_file_open: false,
      is_root: true,
    }
  }

  pub fn set_bytes(mut self, bytes: Vec<u8>) -> Self {
    self.bytes = bytes;
    self
  }

  pub fn set_cwd(mut self, cwd: usize) -> Self {
    self.cwd_fat_entry = cwd;
    self
  }

  pub fn get_cwd(&self) -> usize {
    self.cwd_fat_entry
  }

  pub fn open_file(mut self, filename: String) -> Self {
    // Read file as bytes
    self = self.set_bytes(std::fs::read(&filename).expect("Can't read the image file!"));

    println!("Opened image file!");

    // Set image filename
    self.image_filename = filename;

    // Set flag
    self.is_image_file_open = true;

    self
  }

  pub fn save_file(self) -> Self {
    // Opening a file with truncate will replace contents
    let mut file = OpenOptions::new()
      .write(true)
      .truncate(true)
      .open(&self.image_filename)
      .expect("Can't open the image file!");
    file.write(&self.bytes).expect("Can't write data to file!");

    self
  }

  pub fn is_root(&self) -> bool {
    self.is_root
  }
}
