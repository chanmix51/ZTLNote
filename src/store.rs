use std::fs;
use std::path::{Path, PathBuf};
use std::fmt;
use uuid::Uuid;

use crate::{note::NoteMetaData, error::Result};

/**
This kind of problems raise the impossibility to perform the task because of
physical layer. Keep in mind that theses errors are caught and they make the
program to *panic* so use them with care with a nice & meaningful error message.
 */
#[derive(Debug)]
pub struct StoreError {
    message: String,
}

impl StoreError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "STORE I/O ERROR → {}", self.message)
    }
}

impl std::error::Error for StoreError {}


/**
IOStore declares all the functions a Store needs to perform to a physical IO
subsystem to manage the Zettenkasten organization
 */
pub trait IOStore {
    fn get_fields(&self) -> Result<Vec<String>>;
    fn create_field(&self, field: &str) -> Result<()>;
    fn set_current_field(&self, field: &str) -> Result<()>;
    fn get_current_field(&self) -> Result<Option<String>>;
    fn field_exists(&self, field: &str) -> bool;

    fn get_paths(&self, field: &str) -> Result<Vec<String>>;
    fn get_path(&self, field: &str, path: &str) -> Result<Uuid>;
    fn write_path(&self, field: &str, path: &str, uuid: Uuid) -> Result<()>;
    fn path_exists(&self, field: &str, path: &str) -> bool;
    fn set_current_path(&self, field: &str, path: &str) -> Result<()>;
    fn get_current_path(&self, field: &str) -> Result<Option<String>>;

    fn add_note(&self, field: &str, path: &str, filename: &str) -> Result<NoteMetaData>;
    fn update_note_content(&self, filename: &str, note_id: Uuid) -> Result<()>;
    fn get_note_metadata(&self, uuid: Uuid) -> Result<Option<NoteMetaData>>;
}

#[derive(Debug)]
pub struct Store<'a> {
    base_dir: &'a str,
}

impl<'a> Store<'a> {
    pub fn init(base_dir: &'a str) -> Result<Self> {
        let path = Path::new(base_dir);
        if path.exists() {
            return Err(From::from(StoreError::new(format!("Given directory '{}' already exists.", base_dir))));
        }
        fs::create_dir_all(base_dir)?;
        fs::create_dir(path.join("meta"))?;
        fs::create_dir(path.join("notes"))?;
        fs::create_dir(path.join("fields"))?;
        fs::File::create(path.join("index"))?;

        Ok(Self { base_dir })
    }

    pub fn attach(base_dir: &'a str) -> Result<Self> {
        let path = Path::new(base_dir);
        if !path.is_dir() {
            return Err(From::from(StoreError::new(format!("Given path '{}' is not a directory.", base_dir))));
        }

        if !(
            path.join("meta").is_dir()
            && path.join("notes").is_dir()
            && path.join("index").is_file()
            && path.join("fields").is_dir()
            ) {
            return Err(From::from(StoreError::new(format!("Invalid ztln structure in dir '{}'.", base_dir))))
        }

        Ok( Self { base_dir })
    }

    fn get_basedir_pathbuf(&self) -> PathBuf {
        PathBuf::new().join(self.base_dir)
    } 

    fn get_field_pathbuf(&self, field: &str) -> PathBuf {
      self.get_basedir_pathbuf()
        .join("fields")
        .join(field)
    }

    fn get_path_pathbuf(&self, field: &str, path: &str) -> PathBuf {
      self.get_basedir_pathbuf()
        .join("fields")
        .join(field)
        .join("paths")
        .join(path)
    }
}

impl<'a> IOStore for Store<'a> {
    fn get_current_field(&self) -> Result<Option<String>> {
        let pathbuf = self.get_basedir_pathbuf().join("_CURRENT");

        Ok(if pathbuf.is_file() { Some(fs::read_to_string(pathbuf)?) } else { None })
    }

    fn get_fields(&self) -> Result<Vec<String>> {
        let path = self.get_basedir_pathbuf().join("fields");
        let mut fields = Vec::new();

        for entry in fs::read_dir(path)? {
            let filename = entry?.file_name().to_str().unwrap_or("").to_string();
            if !filename.is_empty() {
                fields.push(filename);
            }
        }
        fields.sort();

        Ok(fields)
    }

    fn create_field(&self, field: &str) -> Result<()> {
        fs::create_dir_all(self.get_field_pathbuf(field).join("paths"))?;

        Ok(())
    }

    fn set_current_field(&self, field: &str) -> Result<()> {
        let file_path = self.get_basedir_pathbuf().join("_CURRENT");
        fs::write(file_path, field)?;

        Ok(())
    }

    fn field_exists(&self, field: &str) -> bool {
      self.get_field_pathbuf(field).exists()  
    }

    fn get_paths(&self, field: &str) -> Result<Vec<String>> {
        let pathbuf = self.get_field_pathbuf(field).join("paths");
        let mut paths = Vec::new();

        for entry in fs::read_dir(pathbuf)? {
            let filename = entry?.file_name().to_str().unwrap_or("").to_string();
            if !filename.is_empty() {
                paths.push(filename);
            }
        }
        paths.sort();

        Ok(paths)
    }

    fn get_path(&self, field: &str, path: &str) -> Result<Uuid> {
        let uuid = Uuid::parse_str(fs::read_to_string(self.get_path_pathbuf(field, path))?.as_str())?;

        Ok(uuid)
    }

    fn write_path(&self, field: &str, path: &str, uuid: Uuid) -> Result<()> {
        fs::write(self.get_path_pathbuf(field, path), uuid.to_string())?;
        
        Ok(())
    }

    fn set_current_path(&self, field: &str, path: &str) -> Result<()> {
        let pathbuf = self.get_field_pathbuf(field).join("_HEAD");
        fs::write(pathbuf, path)?;

        Ok(())
    }

    fn get_current_path(&self, field: &str) -> Result<Option<String>> {
        let pathbuf = self.get_field_pathbuf(field).join("_HEAD");
        if pathbuf.exists() {
            Ok(Some(fs::read_to_string(pathbuf)?))
        } else {
            Ok(None)
        }
    }

    fn path_exists(&self, field: &str, path: &str) -> bool {
        self.get_path_pathbuf(field, path).exists()  
    }

    fn update_note_content(&self, filename: &str, note_id: Uuid) -> Result<()> {
        let target_path = self.get_basedir_pathbuf().join("notes").join(note_id.to_string());
        fs::copy(filename, target_path)?;

        Ok(())
    }

    fn add_note(&self, field: &str, path: &str, filename: &str) -> Result<NoteMetaData> {
        let note_id = Uuid::new_v4();
        let note_id_str = note_id.to_string();
        let note_target_path = self.get_basedir_pathbuf().join("meta").join(note_id_str);
        let parent_id = self.get_path(field, path).ok();
        let metadata = NoteMetaData {
            note_id,
            parent_id,
            references: Vec::new(),
        };
        self.write_path(field,path, note_id)?;
        fs::write(note_target_path, metadata.serialize())?;
        self.update_note_content(filename, note_id)?;
            
        Ok(metadata)
    }

    fn get_note_metadata(&self, _uuid: Uuid) -> Result<Option<NoteMetaData>> {
        Err(From::from("get_note: UNIMPLEMENTED"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        let base_dir = "tmp/ztln_store1";
        let path = Path::new(base_dir);
        let _store = Store::init(base_dir).unwrap();
        assert!(Store::init(base_dir).is_err());
        assert!(path.join("fields").is_dir());
        assert!(path.join("meta").is_dir());
        assert!(path.join("notes").is_dir());
        assert!(path.join("index").is_file());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn create_field() {
        let base_dir = "tmp/ztln_store2";
        let store = Store::init(base_dir).unwrap();
        let path = Path::new(base_dir);
        assert!(!path.join("fields").join("fieldA").exists());
        store.create_field("fieldA").unwrap();
        assert!(path.join("fields").join("fieldA").is_dir());
        assert!(!path.join("fields").join("fieldA").join("HEAD").exists());
        assert!(path.join("fields").join("fieldA").join("paths").is_dir());
        assert!(!path.join("fields").join("fieldA").join("paths").join("main").exists());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn set_current_field() {
        let base_dir = "tmp/ztln_store3";
        let pathbuf = Path::new(base_dir);
        let store = Store::init(base_dir).unwrap();
        store.create_field("fieldA").unwrap();
        assert!(!pathbuf.join("_CURRENT").exists());
        assert!(store.set_current_field("fieldA").is_ok());
        assert_eq!(fs::read_to_string(pathbuf.join("_CURRENT")).unwrap(), "fieldA");

        fs::remove_dir_all(base_dir).unwrap();
    }

    #[test]
    fn get_fields() {
        let base_dir = "tmp/ztln_store4";
        let store = Store::init(base_dir).unwrap();
        assert_eq!(0, store.get_fields().unwrap().len(), "return an empty list of fields");
        store.create_field("fieldB").unwrap();
        assert_eq!(vec!["fieldB"], store.get_fields().unwrap(), "one field");
        store.create_field("fieldA").unwrap();
        assert_eq!(vec!["fieldA", "fieldB"], store.get_fields().unwrap(), "two fields sorted by alphabetical order");

        fs::remove_dir_all(base_dir).unwrap();
    }

    #[test]
    fn add_note() {
        let base_dir = "tmp/ztln_store5";
        let base_dir_path = Path::new(base_dir);
        let store = Store::init(base_dir).unwrap();
        store.create_field("fieldA").unwrap();
        store.set_current_field("fieldA").unwrap();
        let draft_note_path = Path::new("tmp/test5");
        fs::write(draft_note_path, "This is a note").unwrap();
        let result = store.add_note("fieldA", "main", "tmp/test5");
        assert!(result.is_ok(), "adding a note returns OK");
        let note = result.unwrap();
        assert!(note.parent_id.is_none(), "when a field is new, there is no parent_id");
        assert_eq!(note.note_id.to_string(), fs::read_to_string(base_dir_path.join("fields/fieldA/paths/main")).unwrap(), "path has been updated");
        assert!(base_dir_path.join("meta").join(note.note_id.to_string()).is_file(), "meta file exists");
        assert_eq!("This is a note", fs::read_to_string(base_dir_path.join("notes").join(note.note_id.to_string())).unwrap(), "content file is up to date");
        fs::write(draft_note_path, "This is another note").unwrap();
        let another_note = store.add_note("fieldA", "main", "tmp/test5").unwrap();
        assert_eq!(Some(note.note_id), another_note.parent_id, "new note relates to parent");
        assert_eq!(another_note.note_id.to_string(), fs::read_to_string(base_dir_path.join("fields/fieldA/paths/main")).unwrap(), "path has been updated");

        fs::remove_dir_all(base_dir).unwrap();
    }
}