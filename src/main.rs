use ztln::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct MainOpt {
    #[structopt(long, env="ZTLN_BASE_DIR")]
    base_dir: String,
    #[structopt(subcommand)]
    command: MainCommand,
}

impl MainOpt {
    fn execute(&self) -> Result<()> {
        self.command.execute(&(self.base_dir))
    }
}

#[derive(Debug, StructOpt)]
enum MainCommand {
    #[structopt(about="obtain information about an organization")]
    Info(InfoCommand),
}

impl MainCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        match self {
            MainCommand::Info(cmd) => cmd.execute(base_dir),
        }
    }
}

#[derive(Debug, StructOpt)]
struct InfoCommand {}

impl InfoCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        let mut orga = Organization::new(Store::attach(base_dir)?);
        println!("Organization located at: {}", base_dir);
        let current_field = orga.get_current_field();
        if let Some(field) = current_field {
            let field = field;
            println!("Current field: {}", &field);
            println!("Current path: {}", orga.get_current_path(&field)?.unwrap_or_else(|| "None".to_string()));
        } else {
            println!("Current field: None");
            println!("Current path: None");
            println!("Use `ztln field create` to create a new field.");
        }
        Ok(())
    }
}

fn main() {
    MainOpt::from_args()
        .execute()
        .unwrap_or_else(|e| { eprintln!("ERROR: {}", e); std::process::exit(1); });
}