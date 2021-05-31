use std::convert::TryFrom;
use std::{
    env,
    fs::{self, File, Metadata},
    io::{Error, Read},
    path::Path,
    process,
};

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Not enough arguments\n");
        usage();
        process::exit(exitcode::USAGE);
    }

    let mode_request = &args[1];
    match mode_request.as_str() {
        "-tex0" => to_tex0(args),
        "-gct" => to_gct(args),
        _ => {
            println!("Invalid operating mode.\n");
            usage();
            process::exit(exitcode::USAGE);
        }
    }
}

fn usage() {
    println!("Usage:");
    println!("gctconv -tex0 file.gct");
    println!("gctconv -gct file.tex0");
}

fn to_tex0(args: Vec<String>) {
    let file_path_string = &args[2];
    let path = Path::new(file_path_string);
    let filename = path.file_name();
    let fn_string;
    match filename {
        Some(fname) => match fname.to_str() {
            Some(str) => fn_string = str,
            None => fn_string = "[filename error]",
        },
        None => fn_string = "[filename error]",
    }

    let md;
    let md_result = fs::metadata(path);
    match md_result {
        Ok(m) => md = m,
        Err(error) => {
            let error_string = error.to_string();
            println!("GCT Metadata Error: {}\n", error_string);
            usage();
            process::exit(exitcode::NOINPUT);
        }
    }
    let filesize = md.len();

    let fs_int;
    let fs_int_result = u32::try_from(filesize);
    match fs_int_result {
        Ok(i) => fs_int = i,
        Err(error) => {
            let error_string = error.to_string();
            println!("GCT Too Big: {}\n", error_string);
            usage();
            process::exit(exitcode::NOINPUT);
        }
    }

    let gct = File::open(path);
    let mut gct_file;
    match gct {
        Ok(f) => gct_file = f,
        Err(error) => {
            let error_string = error.to_string();
            println!("GCT Error: {}\n", error_string);
            usage();
            process::exit(exitcode::NOINPUT);
        }
    }

    let mut width_bytes = [0; 2];
    let width_result = gct_file.read_exact(&mut width_bytes);
    match width_result {
        Ok(_) => {}
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to read width: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    let mut height_bytes = [0; 2];
    let height_result = gct_file.read_exact(&mut height_bytes);
    match height_result {
        Ok(_) => {}
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to read height: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    let mut enc_byte = [0; 1];
    let enc_result = gct_file.read_exact(&mut enc_byte);
    match enc_result {
        Ok(_) => {}
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to read encoding type: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }
}

fn to_gct(args: Vec<String>) {}

fn read_short(mut file: &File) -> Result<u16, Error> {
    let mut buffer = [0; 2]; // 2 byte buffer
    file.read_exact(&mut buffer)?;

    let s = u16::from_be_bytes(buffer);
    Ok(s)
}
