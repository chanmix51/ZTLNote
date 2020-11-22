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
    #[structopt(about="manage topics")]
    Topic(TopicCommand),
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
            MainCommand::Topic(cmd) => cmd.execute(base_dir),
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
        let current_topic = orga.get_current_topic();
        if let Some(topic) = current_topic {
            let topic = topic;
            println!("Current topic: {}", &topic);
            println!("Current path: {}", orga.get_current_path(&topic)?.unwrap_or_else(|| "None".to_string()));
        } else {
            println!("Current topic: None");
            println!("Current path: None");
            println!("Use `ztln topic create` to create a new topic.");
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
enum TopicCommand {
    #[structopt(about="create a new topic")]
    Create(CreateTopicCommand), 
    List(ListTopicCommand),
    Default(DefaultTopicCommand),
}

impl TopicCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        let mut orga = Organization::new(Store::attach(base_dir)?);
        match self {
            TopicCommand::Create(cmd) => cmd.execute(&mut orga),
            TopicCommand::List(cmd) => cmd.execute(&mut orga),
            TopicCommand::Default(cmd) => cmd.execute(&mut orga),
        }
    }
}

#[derive(Debug, StructOpt)]
struct CreateTopicCommand {
    topic_name: String
}
impl CreateTopicCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        orga.create_topic(&self.topic_name)
    }
}

#[derive(Debug, StructOpt)]
struct ListTopicCommand {}

impl ListTopicCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        let list = orga.get_topics_list();
        if list.is_empty() {
            println!("No topics.");
        } else {
            let current = orga.get_current_topic().unwrap_or_else(|| "".to_string());
            for topic in list {
                println!("{} {}", if topic == current { "→" } else { " " }, topic);
            }
        }
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct DefaultTopicCommand {
    topic_name: String,
}

impl DefaultTopicCommand {
    pub fn execute(&self, orga: &mut Organization) -> Result<()> {
        orga.set_current_topic(&self.topic_name)
    }
}

#[derive(Debug, StructOpt)]
struct PathCommand {
    #[structopt(help="the name of the topic containing the paths")]
    topic: Option<String>,
    #[structopt(subcommand)]
    subcommand: SubPathCommand,
}
#[derive(Debug, StructOpt)]
enum SubPathCommand {
    #[structopt(about="list the paths for a given topic")]
    List(ListPathCommand),
    Create(CreatePathCommand),
}

impl PathCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        let mut orga = Organization::new(Store::attach(base_dir)?);
        match &self.subcommand {
            SubPathCommand::List(cmd)
                => cmd.execute(&mut orga, self.topic.clone()),
            SubPathCommand::Create(cmd)
                => cmd.execute(&mut orga, self.topic.clone()),
        }
    }
}

#[derive(Debug, StructOpt)]
struct ListPathCommand {
}

impl ListPathCommand {
    fn execute(&self, orga: &mut Organization, topic: Option<String>) -> Result<()> {
        let (topic, list) = orga.get_paths_list(topic.as_deref())?;
        if list.is_empty() {
            println!("No paths in topic '{}'.", topic);
        } else {
            let current = orga.get_current_path(&topic)?.unwrap_or_else(|| "".to_string());
            for path in list {
                println!("{} {}", if path == current { "→" } else { " " }, path);
            }
        }
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct CreatePathCommand {
    #[structopt(help="name of the new path")]
    new_path: String,
    #[structopt(long, short, help="branch from this path instead of current path")]
    path: Option<String>,
}

impl CreatePathCommand {
    fn execute(&self, orga: &mut Organization, topic: Option<String>) -> Result<()> {
        orga.create_path(topic.as_deref(), &self.new_path, self.path.as_deref())?;
        
        Ok(())
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
    #[structopt(long,short,help="set the current topic prior to add the note")]
    topic: Option<String>,
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
        let r = orga.add_note(&filename, self.topic.as_deref(), self.path.as_deref())?;
        let note_id = r.note_id.to_string();
        let parent_id = r.parent_id.map_or_else(|| "".to_string(), |v| v.to_string());
        println!("Note '{}' ← '{}' added at {}/{}", parent_id, note_id, r.topic, r.path);
        std::fs::remove_file(filename)?;

        Ok(())
    }
}

fn main() {
    MainOpt::from_args()
        .execute()
        .unwrap_or_else(|e| { eprintln!("ERROR: {}", e); std::process::exit(1); });
}