use actix::{Actor, Context, Handler, Message};
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::Result;

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
                self.0.set_style(ProgressStyle::default_spinner()
                    .template("[{elapsed_precise}] \x1b[32m{msg}\x1b[0m"));
                self.0.finish_with_message(final_message);
            }
        }
        Ok(())
    }
}
