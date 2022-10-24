use crate::runner;

#[derive(clap::Args)]
pub struct Args {
    #[clap(long)]
    filter: Option<String>,
    #[clap(long, takes_value = false)]
    debug: bool,
    #[clap(long, takes_value = false)]
    update_expected: bool,
}

pub fn exec(cmd: Args) {
    let runner = runner::Runner::load(crate::TEST_SUITES_PATH, crate::EXAMPLES_PATH);
    let config =
        runner::Config { filter: cmd.filter.unwrap_or_else(|| "*".to_owned()), debug: cmd.debug };
    let res = runner.run(&config);
    if cmd.update_expected {
        res.update_expected();
        println!("Updated expected outputs.");
    } else {
        res.print();
    }
    if !res.success() {
        std::process::exit(1);
    }
}