use okra::boxchecker::BoxMaker;
use okra::sqlite_boxchecker::SqliteBoxes;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "create-action",
    about = "Insert a new action into the configuration."
)]
struct Opt {
    #[structopt(short, long)]
    action_name: String,

    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let mut boxer = SqliteBoxes::new(opt.file.as_os_str().to_str().unwrap());
    let id = boxer.create_action(&opt.action_name);
    println!("{}", id);
}
