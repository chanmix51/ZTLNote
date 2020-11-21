use ztln::*;
use structopt::StructOpt;
use std::process::Command;
use rand::Rng; 
use rand::distributions::Alphanumeric;
use std::env;

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
    #[structopt(about="manage paths")]
    Path(PathCommand),
    #[structopt(about="add or update notes")]
    Note(NoteCommand),
}

impl MainCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        match self {
            MainCommand::Info(cmd) => cmd.execute(base_dir),
            MainCommand::Init(cmd) => cmd.execute(base_dir),
            MainCommand::Field(cmd) => cmd.execute(base_dir),
            MainCommand::Path(cmd) => cmd.execute(base_dir),
            MainCommand::Note(cmd) => cmd.execute(base_dir),
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

#[derive(Debug, StructOpt)]
struct PathCommand {
    #[structopt(help="the name of the field containing the paths")]
    field: Option<String>,
    #[structopt(subcommand)]
    subcommand: SubPathCommand,
}
#[derive(Debug, StructOpt)]
enum SubPathCommand {
    #[structopt(about="list the paths for a given field")]
    List(ListPathCommand),
    Create(CreatePathCommand),
}

impl PathCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        let mut orga = Organization::new(Store::attach(base_dir)?);
        let field = if let Some(f) = self.field.clone() {
            f
        } else {
            orga.get_current_field()
                .ok_or_else(|| ZtlnError::Default("No field given.".to_string()))?
        };
        match &self.subcommand {
            SubPathCommand::List(cmd)
                => cmd.execute(&mut orga, &field),
            SubPathCommand::Create(cmd)
                => cmd.execute(&mut orga, &field),
        }
    }
}

#[derive(Debug, StructOpt)]
struct ListPathCommand {
}

impl ListPathCommand {
    fn execute(&self, orga: &mut Organization, field: &str) -> Result<()> {
        let list = orga.get_paths_list(&field);
        if list.is_empty() {
            println!("No paths in field '{}'.", field);
        } else {
            let current = orga.get_current_path(&field)?.unwrap_or_else(|| "".to_string());
            for path in list {
                println!("{} {}", if path == current { "→" } else { " " }, path);
            }
        }
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct CreatePathCommand {}

impl CreatePathCommand {
    fn execute(&self, orga: &mut Organization, field: &str) -> Result<()> {
        Err(From::from("CreatePathCommand::execute NOT IMPLEMENTED"))
    }
}

#[derive(Debug, StructOpt)]
enum NoteCommand {
    #[structopt(about="Add a new note")]
    Add(AddNoteCommand),
}

impl NoteCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        let mut orga = Organization::new(Store::attach(base_dir)?);
        match self {
            NoteCommand::Add(cmd) => cmd.execute(&mut orga),
        }
    }
}

#[derive(Debug, StructOpt)]
struct AddNoteCommand {
    filename: Option<String>,
    #[structopt(long,short,help="set the current field prior to add the note")]
    field: Option<String>,
    #[structopt(long,short,help="set the current path prior to add the note")]
    path: Option<String>,
}

impl AddNoteCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        let filename = match self.filename.as_ref() {
            Some(f) => f.clone(),
            None => {
                let pathbuf = env::temp_dir().join(rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(10)
                    .collect::<String>()); 
                let f = pathbuf.to_str().unwrap();
                Command::new(env::var_os("EDITOR").unwrap_or_else(|| From::from("vi".to_string())))
                    .arg(f)
                    .status()?;
                f.to_string()
            }
        };
        let r = orga.add_note(&filename, self.field.as_deref(), self.path.as_deref())?;
        let note_id = r.note_id.to_string();
        let parent_id = r.parent_id.map_or_else(|| "".to_string(), |v| v.to_string());
        println!("Note '{}' ← '{}' added at {}/{}", parent_id, note_id, r.field, r.path);
        std::fs::remove_file(filename)?;

        Ok(())
    }
}

fn main() {
    MainOpt::from_args()
        .execute()
        .unwrap_or_else(|e| { eprintln!("ERROR: {}", e); std::process::exit(1); });
}