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
    #[structopt(about="initialize a new organization")]
    Init(InitCommand),
    #[structopt(about="manage fields")]
    Field(FieldCommand),
}

impl MainCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        match self {
            MainCommand::Info(cmd) => cmd.execute(base_dir),
            MainCommand::Init(cmd) => cmd.execute(base_dir),
            MainCommand::Field(cmd) => cmd.execute(base_dir),
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

#[derive(Debug, StructOpt)]
struct InitCommand {}

impl InitCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        Store::init(base_dir).map(|_| ())
    }
}

#[derive(Debug, StructOpt)]
enum FieldCommand {
    #[structopt(about="create a new field")]
    Create(CreateFieldCommand), 
    List(ListFieldCommand),
    Default(DefaultFieldCommand),
}

impl FieldCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        let mut orga = Organization::new(Store::attach(base_dir)?);
        match self {
            FieldCommand::Create(cmd) => cmd.execute(&mut orga),
            FieldCommand::List(cmd) => cmd.execute(&mut orga),
            FieldCommand::Default(cmd) => cmd.execute(&mut orga),
        }
    }
}

#[derive(Debug, StructOpt)]
struct CreateFieldCommand {
    field_name: String
}
impl CreateFieldCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        orga.create_field(&self.field_name)
    }
}

#[derive(Debug, StructOpt)]
struct ListFieldCommand {}

impl ListFieldCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        let list = orga.get_fields_list();
        if list.is_empty() {
            println!("No fields.");
        } else {
            let current = orga.get_current_field().unwrap_or_else(|| "".to_string());
            for field in list {
                println!("{} {}", if field == current { "→" } else { " " }, field);
            }
        }
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct DefaultFieldCommand {
    field_name: String,
}

impl DefaultFieldCommand {
    pub fn execute(&self, orga: &mut Organization) -> Result<()> {
        orga.set_current_field(&self.field_name)
    }
}

fn main() {
    MainOpt::from_args()
        .execute()
        .unwrap_or_else(|e| { eprintln!("ERROR: {}", e); std::process::exit(1); });
}