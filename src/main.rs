use std::env;

mod parse;

fn main() {
    //vector for cmdl args
    let args: Vec<String> = env::args().collect();

    //if not exactly 3 cmdl args error and exit return code
    if args.len() != 3 {
        eprintln!("Usage: ./cfg_generator <binary_path> <memory_address>");
        std::process::exit(1);
    }
    let binary_path = &args[1];
    let virtual_address = u64::from_str_radix(&args[2].strip_prefix("0x").unwrap(), 16).unwrap();

    parse::generate_cfg(binary_path, &virtual_address);
}
