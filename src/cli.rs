use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub config_path: Option<PathBuf>,
    pub log_level: Option<String>,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            config_path: None,
            log_level: None,
        }
    }
}

/// 解析命令行参数
pub fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();

    let mut cli = CliArgs::default();

    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--config" | "-c" => {
                if i + 1 < args.len() {
                    cli.config_path = Some(PathBuf::from(&args[i + 1]));
                }
            }
            "--log-level" => {
                if i + 1 < args.len() {
                    cli.log_level = Some(args[i + 1].clone());
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-v" => {
                println!("TUI Workstation v{} (WIP)", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => {}
        }
    }

    cli
}

fn print_help() {
    println!("TUI Workstation - Terminal UI Workstation");
    println!();
    println!("USAGE:");
    println!("    tui-workstation [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -c, --config <PATH>     Path to config file");
    println!("        --log-level <LEVEL> Log level (trace, debug, info, warn, error)");
    println!("    -h, --help              Print help");
    println!("    -v, --version           Print version");
}
