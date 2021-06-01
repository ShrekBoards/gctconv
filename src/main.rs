use std::{
    convert::{TryFrom, TryInto},
    io::{Seek, SeekFrom},
};
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
    let fn_no_ext = path.file_stem();
    let fn_string;
    let fs_string;
    match filename {
        Some(fname) => match fname.to_str() {
            Some(str) => fn_string = str,
            None => fn_string = "[filename error]",
        },
        None => fn_string = "[filename error]",
    }

    match fn_no_ext {
        Some(fname) => match fname.to_str() {
            Some(str) => fs_string = str,
            None => fs_string = "[filename error]",
        },
        None => fs_string = "[filename error]",
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

    let mut fs_int;
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

    let seek_result = gct_file.seek(SeekFrom::Start(0x10));
    match seek_result {
        Ok(_) => {}
        Err(error) => {
            let error_string = error.to_string();
            println!("Seek Error: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
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

    let seek_result = gct_file.seek(SeekFrom::Start(0x40));
    match seek_result {
        Ok(_) => {}
        Err(error) => {
            let error_string = error.to_string();
            println!("Seek Error: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    let mut rest_of_file = Vec::new();
    let read_result = gct_file.read_to_end(&mut rest_of_file);
    match read_result {
        Ok(_) => {}
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to read rest of file: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    let tex0_ascii = "TEX0";
    if !tex0_ascii.is_ascii() {
        println!("\"{}\" isn't ascii\n", tex0_ascii);
        usage();
        process::exit(exitcode::DATAERR);
    }

    // header starts with "TEX0"
    let mut header = tex0_ascii.as_bytes().to_owned();
    // header == ["T", "E", "X", "0"]

    // then the filesize, big endian
    if enc_byte[0] == 0x08 {
        // if CI4, take 0x20 off the total
        fs_int -= 0x20;
    }
    let fs_bytes = fs_int.to_be_bytes().to_vec();
    header.extend(fs_bytes);
    // header == [...[0x04], FS_1, FS_2, FS_3, FS_4]

    // then tex0 version as int, 1 in our case
    let four_byte_1 = 1_u32.to_be_bytes().to_vec();
    header.extend(&four_byte_1);
    // header == [...[0x08], 1_byte_1, 1_byte_2, 1_byte_3, 1_byte_4]

    // then int 0
    let four_byte_0 = 0_u32.to_be_bytes().to_vec();
    header.extend(&four_byte_0);
    // header == [...[0x0C], 0_byte, 0_byte, 0_byte, 0_byte]

    // then 'idk'?? just hex 0x40 as int
    let idk = 0x40_u32.to_be_bytes().to_vec();
    header.extend(idk);
    // header == [...[0x10], 0x40_1, 0x40_2, 0x40_3, 0x40_4]

    // then 0x4 + filesize, as int
    let fs_plus_4 = fs_int + 0x4;
    let fs_p4_bytes = fs_plus_4.to_be_bytes().to_vec();
    header.extend(fs_p4_bytes);
    // header == [...[0x14], FS_P4_1, FS_P4_2, FS_P4_3, FS_P4_4]

    if enc_byte[0] == 0x08 {
        // int 1 if 0x08 (CI4)
        header.extend(&four_byte_1);
    } else {
        // otherwise int 0
        header.extend(&four_byte_0);
    }
    // header == [...[0x18], 0/1_byte, 0/1_byte, 0/1_byte, 0/1_byte]

    // width as short, height as short, enc as byte
    header.extend(width_bytes.to_vec());
    // header == [...[0x1A], width_1, width_2]
    header.extend(height_bytes.to_vec());
    // header == [...[0x1C], height_1, height_2]

    // padding for enc byte
    header.extend([0; 3].to_vec());
    // header == [...[0x20], 0_byte, 0_byte, 0_byte]

    // encoding byte
    header.extend(enc_byte.to_vec());
    // header == [...[0x23], enc_byte]

    // mipmap count + 1 as int; no mipmaps, so just 1
    header.extend(&four_byte_1);
    // header == [...[0x24], 1_byte_1, 1_byte_2, 1_byte_3, 1_byte_4]

    let padding = [0_u8; 24].to_vec();
    header.extend(padding);
    // header is now padded to 0x40

    let mut plt0_file = if enc_byte[0] == 0x08 {
        // if CI4, extract palette data
        let palette_data = rest_of_file.split_off(rest_of_file.len() - 0x20);

        const PLT0_HEADER: [u8; 0x40] = [
            0x50, 0x4C, 0x54, 0x30, 0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut plt0_file = PLT0_HEADER.to_vec();
        plt0_file.extend(palette_data);
        plt0_file
    } else {
        Vec::new()
    };

    let mut tex0_file = header;
    tex0_file.extend(rest_of_file);

    let mut footer = vec![0_u8; 3]; // three 0 bytes
    if !fs_string.is_ascii() {
        println!("\"{}\" isn't ascii\n", fs_string);
        usage();
        process::exit(exitcode::DATAERR);
    }

    let stem_bytes = fs_string.as_bytes().to_owned();
    let stem_length = stem_bytes.len();
    let sl_byte;
    let sl_byte_result = u8::try_from(stem_length);
    match sl_byte_result {
        Ok(i) => sl_byte = i,
        Err(error) => {
            let error_string = error.to_string();
            println!("GCT Filename Too Big: {}\n", error_string);
            usage();
            process::exit(exitcode::NOINPUT);
        }
    }

    footer.extend(vec![sl_byte]); // vec.extend_one is unstable, so have to vec![]
    footer.extend(stem_bytes);
    footer.extend([0; 3].to_vec());

    tex0_file.extend(&footer);

    let tex0_path = format!("output/Textures(NW4R)/{}.tex0", fs_string);
    let write_result = fs::write(tex0_path, tex0_file);
    match write_result {
        Ok(_) => {}
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to write TEX0 file: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    if plt0_file.is_empty() {
        return;
    }

    plt0_file.extend(footer);

    let plt0_path = format!("output/Palettes(NW4R)/{}.plt0", fs_string);
    let write_result = fs::write(plt0_path, plt0_file);
    match write_result {
        Ok(_) => {}
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to write PLT0 file: {}\n", error_string);
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
