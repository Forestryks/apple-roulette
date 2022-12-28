use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Apple Roulette")]
#[command(about = "Want some strudel?", long_about = None)]
#[command(author, version)]
pub struct Args {
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[arg(long, default_value_t = 20)]
    pub panic_after: usize,
}
