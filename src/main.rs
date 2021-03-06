use ztln::*;
use structopt::StructOpt;
use std::process::Command;
use rand::Rng; 
use rand::distributions::Alphanumeric;
use std::env;

#[derive(Debug, StructOpt)]
struct MainOpt {
    #[structopt(long, env="ZTLN_BASE_DIR", help="organization directory path")]
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
    #[structopt(about="Obtain information about an organization.")]
    Info(InfoCommand),
    #[structopt(about="Initialize a new organization.")]
    Init(InitCommand),
    #[structopt(about="Manage topics.")]
    Topic(TopicCommand),
    #[structopt(about="Manage paths.")]
    Path(PathCommand),
    #[structopt(about="Add or update notes.")]
    Note(NoteCommand),
    #[structopt(about="Manage tags.")]
    Tag(TagCommand),
}

impl MainCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        match self {
            MainCommand::Info(cmd) => cmd.execute(base_dir),
            MainCommand::Init(cmd) => cmd.execute(base_dir),
            MainCommand::Topic(cmd) => cmd.execute(base_dir),
            MainCommand::Path(cmd) => cmd.execute(base_dir),
            MainCommand::Note(cmd) => cmd.execute(base_dir),
            MainCommand::Tag(cmd) => cmd.execute(base_dir),
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
        Store::init(base_dir)?;
        println!("Ztln version {} organization intialized at '{}'.", env!("CARGO_PKG_VERSION"), base_dir);
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
enum TopicCommand {
    #[structopt(about="create a new topic")]
    Create(CreateTopicCommand), 
    #[structopt(about="list all topics")]
    List(ListTopicCommand),
    #[structopt(about="set the default topic")]
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
    #[structopt(about="branch a new path from either the current path or a given path")]
    Branch(BranchPathCommand),
    #[structopt(about="set the default path in a topic")]
    Default(DefaultPathCommand),
    #[structopt(about="remove a path")]
    Remove(RemovePathCommand),
    #[structopt(about="reset a path to another location")]
    Reset(ResetPathCommand),
}

impl PathCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        let mut orga = Organization::new(Store::attach(base_dir)?);
        match &self.subcommand {
            SubPathCommand::List(cmd)
                => cmd.execute(&mut orga),
            SubPathCommand::Branch(cmd)
                => cmd.execute(&mut orga),
            SubPathCommand::Default(cmd)
                => cmd.execute(&mut orga),
            SubPathCommand::Remove(cmd)
                => cmd.execute(&mut orga),
            SubPathCommand::Reset(cmd)
                => cmd.execute(&mut orga),
        }
    }
}

#[derive(Debug, StructOpt)]
struct DefaultPathCommand {
    #[structopt(short, long, about="use this topic instead of the current one")]
    topic: Option<String>,
    #[structopt(about="new default path")]
    path: String,
}

impl DefaultPathCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        orga.set_current_path(self.topic.as_deref(), &self.path)?;
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct ListPathCommand {
    #[structopt(long, short, about="use this topic instead of the current one")]
    topic: Option<String>,
}

impl ListPathCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        let (topic, list) = orga.get_paths_list(self.topic.as_deref())?;
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
struct BranchPathCommand {
    #[structopt(help="name of the new path")]
    new_path: String,
    #[structopt(long, short, help="branch from this location instead of current HEAD")]
    location: Option<String>,
}

impl BranchPathCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        orga.create_path(&self.new_path, self.location.as_deref())?;
        
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct RemovePathCommand {
    #[structopt(help="the name of the path")]
    path: String,
    #[structopt(short, long, help="the name of the topic if not the default one")]
    topic: Option<String>,
}

impl RemovePathCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        let metadata = orga.remove_path(&self.path, self.topic.as_deref())?;
        println!("path '{}' deleted ({})", self.path, metadata.note_id.to_string()[..8].to_string());

        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct ResetPathCommand {
    #[structopt(help="the name of the path")]
    path: String,
    #[structopt(help="the new location of the path")]
    location: String,
    #[structopt(short, long, help="the name of the topic if not the default one")]
    topic: Option<String>,
}

impl ResetPathCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        let (old_metadata, new_metadata) = orga.reset_path(&self.path, self.topic.as_deref(), &self.location)?;
        println!(
            "path {} reset at {} (was {})",
            self.path,
            &old_metadata.note_id.to_string()[..8],
            &new_metadata.note_id.to_string()[..8]
        );
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
enum NoteCommand {
    #[structopt(about="add a new note")]
    Add(AddNoteCommand),
    #[structopt(about="create a reference to a note")]
    Reference(NoteReferenceCommand),
    #[structopt(about="display a note")]
    Show(NoteShowCommand),
}

impl NoteCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        let mut orga = Organization::new(Store::attach(base_dir)?);
        match self {
            NoteCommand::Add(cmd)
                            => cmd.execute(&mut orga),
            NoteCommand::Reference(cmd)
                            => cmd.execute(&mut orga),
            NoteCommand::Show(cmd)
                            => cmd.execute(&mut orga),
        }
    }
}

#[derive(Debug, StructOpt)]
struct NoteShowCommand {
    location: String,
}

impl NoteShowCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        let metadata = orga.solve_location(&self.location)?
            .ok_or_else(|| ZtlnError::LocationError(self.location.to_string()))?;
        let content = orga.get_note_content(metadata.note_id)?;
        println!("{}", content);
        println!("================================================================================");
        println!("{}", metadata);

        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct NoteReferenceCommand {
    from_location: String,
    to_location: String,
}

impl NoteReferenceCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        orga.add_note_reference(&self.from_location, &self.to_location)?;
        Ok(())
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
        let meta = orga.add_note(&filename, self.topic.as_deref(), self.path.as_deref())?;
        let note_id = meta.note_id.to_string();
        let parent_id = meta.parent_id.map_or_else(|| "".to_string(), |v| v.to_string());
        println!("Note '{}' ← '{}' added at {}/{}", parent_id, note_id, meta.topic, meta.path);
        std::fs::remove_file(filename)?;

        Ok(())
    }
}

#[derive(Debug, StructOpt)]
enum TagCommand {
    #[structopt(about="add a keyword to a note at given location (or HEAD)")]
    Add(TagAddCommand),
    #[structopt(about="get the list of notes that are associated with the given keyword")]
    Search(TagSearchCommand),
    #[structopt(about="list keywords from the index")]
    List(TagListCommand),
}

impl TagCommand {
    fn execute(&self, base_dir: &str) -> Result<()> {
        let mut orga = Organization::new(Store::attach(base_dir)?);
        match self {
            TagCommand::Add(cmd) => cmd.execute(&mut orga),
            TagCommand::Search(cmd) => cmd.execute(&mut orga),
            TagCommand::List(cmd) => cmd.execute(&mut orga)
        }
    }
}

#[derive(Debug, StructOpt)]
struct TagAddCommand {
    #[structopt(help="the keyword to tag the note with")]
    keyword: String,
    #[structopt(help="note's location (defaults to MAIN)")]
    location: Option<String>,
}

impl TagAddCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        orga.add_keyword(&self.keyword, self.location.as_deref())?;
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct TagSearchCommand {
    #[structopt(help="keyword to search in the index")]
    keyword: String,
}

impl TagSearchCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        let list = orga.search_keyword(&self.keyword);
        for note in &list {
            println!("{}", note.note_id.to_string()[..8].to_string());
        }
        if list.is_empty() {
            println!("{} results found.", list.len());
        } else {
            println!("No result found.");
        }
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct TagListCommand {
}

impl TagListCommand {
    fn execute(&self, orga: &mut Organization) -> Result<()> {
        for (kw, count) in orga.list_keywords() {
            println!("{} ({} notes)", kw, count);
        }

        Ok(())
    }
}

fn main() {
    MainOpt::from_args()
        .execute()
        .unwrap_or_else(|e| { eprintln!("ERROR: {}", e); std::process::exit(1); });
}