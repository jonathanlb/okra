use okra::auth::{Auth, LoginInfo, SqliteAuth};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "add-user", about = "Insert a new user into the database.")]
struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    #[structopt(short, long)]
    password: String,

    #[structopt(short, long)]
    username: String,
}

fn main() {
    let opt = Opt::from_args();
    let mut auth = SqliteAuth::new(opt.file.as_os_str().to_str().unwrap()).unwrap();
    let login = LoginInfo {
        username: &opt.username,
        password: &opt.password,
    };
    auth.add_user(&login).unwrap();
}
