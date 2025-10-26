use clap::{
    Args,
    Parser,
    Subcommand,
};


#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct GameboyArgs {
    #[clap(subcommand)]
    pub run_type: RunType,
}

#[derive(Subcommand, Debug)]
pub enum RunType {
    /// Run the Game Boy with the supplied ROM (.gb) file
    Run(RunCommand),

    /// Run the Game Boy in test mode.
    Test(TestCommand),
}

#[derive(Args, Debug)]
pub struct RunCommand {
    /// Path to the rom (.gb) file you wish to load
    pub rom: String,
}

#[derive(Args, Debug)]
pub struct TestCommand {
    /// Path to the rom (.gb) file you wish to load
    pub rom: String,

    /// Log CPU state to file
    pub log: String,
}