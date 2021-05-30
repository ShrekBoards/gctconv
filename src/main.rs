use std::{env, process};

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

}

fn to_gct(args: Vec<String>) {

}