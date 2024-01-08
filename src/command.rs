use clap::Parser;

#[derive(Parser, Debug, Clone, Default)]
/// IC3
pub struct Args {
    /// input dimacs file
    pub dimacs: String,

    /// verbose
    #[arg(short, default_value_t = false)]
    pub verbose: bool,

    /// random seed
    #[arg(short, long)]
    pub random: Option<usize>,

    /// verify
    #[arg(long, default_value_t = false)]
    pub verify: bool,
}
