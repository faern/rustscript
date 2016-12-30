use std::process::{self, Command, ExitStatus};
use std::path::Path;

use {Result, ResultExt};

pub fn run_script<P: AsRef<Path>>(script_bin_path: P, args: Vec<String>) -> Result<()> {
    let cmd_path = script_bin_path.as_ref();
    let mut command = Command::new(cmd_path);
    if !args.is_empty() {
        println!("Using arguments: {:?}", &args);
        command.args(&args);
    }
    let mut script_process = command.spawn()
        .chain_err(|| "Unable to spawn script")?;
    let script_result = script_process.wait().chain_err(|| "Script crashed")?;
    terminate_like_script(script_result);
}

#[cfg(not(unix))]
fn terminate_like_script(exit_status: ExitStatus) -> ! {
    process::exit(exit_status.code().unwrap())
}

#[cfg(unix)]
fn terminate_like_script(exit_status: ExitStatus) -> ! {
    use std::os::unix::process::ExitStatusExt;
    process::exit(match exit_status.code() {
        Some(i) => i,
        None => exit_status.signal().unwrap(),
    })
}
