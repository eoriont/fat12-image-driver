use std::io::*;

pub fn get_arg(args: &Vec<&str>, argnum: usize) -> Result<String> {
  if args.len() > argnum {
    return Ok(args[argnum].to_owned());
  } else {
    return Err(std::io::Error::new(
      ErrorKind::InvalidInput,
      "Not enough arguments!",
    ));
  }
}
