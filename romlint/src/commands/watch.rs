use crate::args::Args;
use crate::linter::Rules;
use crate::ui::Message;
use std::sync::mpsc::Sender;

pub async fn watch(args: &Args, rules: &Rules, tx: &mut Sender<Message>) {
    todo!()
}
