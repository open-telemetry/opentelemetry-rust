use anyhow::Result;
use xshell::Shell;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let task = std::env::args().nth(1);
    let sh = xshell::Shell::new()?;
    if let Some(cmd) = task.as_deref() {
        let f = match cmd {
            "publish" => xtask::publish::publish,
            _ => print_help,
        };
        f(&sh)?;
    } else {
        print_help(&sh)?;
    }
    Ok(())
}

fn print_help(_sh: &Shell) -> Result<()> {
    eprintln!(
        "Tasks:
  - publish
"
    );
    Ok(())
}
