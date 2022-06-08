use crate::constants::PACKAGE_TOML;
use anyhow::{anyhow, Error, Result};
use colored::Colorize;
use std::path::PathBuf;
use actix::{Actor, Context, Handler, Message};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug)]
pub enum ProgressStatus {
    InProgress(String),
    Done,
}

pub struct SpinnerProgressBar(ProgressBar, String);

impl SpinnerProgressBar {
    pub fn new(final_message: String) -> Self {
        let bar = ProgressBar::new(1);
        bar.set_style(ProgressStyle::default_spinner()
            .template("[{elapsed_precise}] {spinner} {msg}"));
        bar.enable_steady_tick(75);
        Self(bar, final_message)
    }
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct AdvanceProgressBar(pub ProgressStatus);

impl Actor for SpinnerProgressBar {
    type Context = Context<Self>;
}

impl Handler<AdvanceProgressBar> for SpinnerProgressBar {
    type Result = Result<()>;

    fn handle(&mut self, msg: AdvanceProgressBar, _ctx: &mut Context<Self>) -> Self::Result {
        let final_message = self.1.clone();
        match msg.0 {
            ProgressStatus::InProgress(progress_string) => {
                self.0.set_message(progress_string);
            },
            ProgressStatus::Done => {
                self.0.finish_with_message(final_message);
            }
        }
        Ok(())
    }
}

pub fn find_toml(current_path: PathBuf) -> Result<PathBuf> {
    let mut toml_path = current_path.join(PACKAGE_TOML);
    if toml_path.is_file() {
        Ok(toml_path)
    } else if toml_path.pop() && toml_path.pop() {
        Ok(find_toml(toml_path)?)
    } else {
        Err(anyhow!("Could not find package.toml"))
    }
}

pub fn print_success_message(message: &str) {
    println!("{} {}", "Success:".green(), message);
}

pub fn print_error_message(error: Error) {
    eprintln!("{} {}", "Error:".red(), error);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tempfile::{Builder, TempDir};

    #[test]
    fn successfully_found_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&temp_dir)?;

        assert!(find_toml(temp_dir.path().to_path_buf())?.is_file());
        package_toml.close()?;
        temp_dir.close()?;
        Ok(())
    }
}
