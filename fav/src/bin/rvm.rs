/// rvm — Favnir standalone bytecode executor.
///
/// Runs `.fvc` bytecode artifacts without the full `fav` toolchain.
/// Suitable for production executor images (ECS / EKS / Lambda) that
/// only need the VM and not the compiler or type-checker.
///
/// Usage:
///   rvm --version           Print VM version and exit
///   rvm --help              Print help and exit
///   rvm [--db <url>] <file.fvc>   Execute bytecode file
use fav_core::backend::vm::VM_VERSION;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        eprintln!("Usage: rvm [--version] [--help] [--db <url>] <file.fvc>");
        std::process::exit(1);
    }

    let mut db_url: Option<String> = None;
    let mut file: Option<String> = None;
    let mut i = 1usize;

    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("Favnir VM {}", VM_VERSION);
                return;
            }
            "--help" | "-h" => {
                println!("rvm — Favnir standalone VM executor");
                println!();
                println!("USAGE:");
                println!("    rvm [--version] [--help] [--db <url>] <file.fvc>");
                println!();
                println!("OPTIONS:");
                println!("    --version    Print VM version and exit");
                println!("    --db <url>   Database connection URL");
                println!("    --help       Print this help message");
                println!();
                println!("ARGS:");
                println!("    <file.fvc>   Bytecode file to execute");
                return;
            }
            "--db" => {
                i += 1;
                if i < args.len() {
                    db_url = Some(args[i].clone());
                } else {
                    eprintln!("rvm: --db requires a URL argument");
                    std::process::exit(1);
                }
            }
            arg if !arg.starts_with('-') => {
                file = Some(arg.to_string());
            }
            unknown => {
                eprintln!("rvm: unknown option `{}`", unknown);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let file = match file {
        Some(f) => f,
        None => {
            eprintln!("rvm: no bytecode file specified");
            eprintln!("Usage: rvm [--version] [--help] [--db <url>] <file.fvc>");
            std::process::exit(1);
        }
    };

    fav_core::exec_fvc_file(&file, db_url.as_deref());
}
