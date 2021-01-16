use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(
    name = "Import visualiser",
    about = "Construct a visualised graph from python code"
)]
pub struct Opt {

    #[structopt(
        name = "Root directory",
        parse(from_os_str),
        about = "Filepath: Root init folder"
    )]
    pub config: PathBuf,
}
