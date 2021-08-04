use okra::boxchecker::{ActionId, BoxMaker};
use okra::sqlite_boxchecker::SqliteBoxes;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "link-action",
    about = "Order actions in a hierarchy in the configuration."
)]
struct Opt {
    #[structopt(short, long)]
    child_action: ActionId,

    #[structopt(parse(from_os_str))]
    file: PathBuf,

    #[structopt(short, long)]
    parent_action: ActionId,
}

fn main() {
    let opt = Opt::from_args();
    let mut boxer = SqliteBoxes::new(opt.file.as_os_str().to_str().unwrap());
    boxer.make_action_parent_of(opt.parent_action, opt.child_action);
}
