use std::{
    env,
    error::Error,
    ffi::OsString,
    fs,
    io::{self, Read},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);

    let input = match (args.next(), args.next()) {
        (None, _) => read_stdin()?,
        (Some(path), None) if path == OsString::from("-") => read_stdin()?,
        (Some(path), None) => fs::read_to_string(path)?,
        (Some(_), Some(_)) => {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "usage: jsonparser [FILE|-]").into(),
            );
        }
    };

    let tokens = jsonparser::tokenise(&input)?;

    let value = jsonparser::parse(tokens)?;

    println!("{value}");

    Ok(())
}

/// Reads all JSON input from standard input.
///
/// # Errors
///
/// Returns any I/O error reported while reading from standard input.
fn read_stdin() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
