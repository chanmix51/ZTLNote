use std::fs;
use std::path::{Path, PathBuf};
use std::fmt;

use crate::error::Result;

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
    fn get_fields(&self) -> Vec<String>;
    fn create_field(&self, field: &str) -> Result<()>;
    fn set_current_field(&self, field: &str) -> Result<()>;
    fn get_current_field(&self) -> Result<Option<String>>;
    fn field_exists(&self, field: &str) -> bool;

    fn get_paths(&self, field: &str) -> Vec<String>;
    fn create_path(&self, field: &str, path: &str) -> Result<()>;
    fn set_current_path(&self, field: &str, path: &str) -> Result<()>;
    fn get_current_path(&self, field: &str) -> Result<Option<String>>;
    fn path_exists(&self, field: &str, path: &str) -> bool;
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
        fs::File::create(path.join("fields/_CURRENT"))?;

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
            && path.join("fields/_CURRENT").is_file()
            ) {
            return Err(From::from(StoreError::new(format!("Invalid ztln structure in dir '{}'.", base_dir))))
        }

        Ok( Self { base_dir })
    }

    fn get_basedir_pathbuf(&self) -> PathBuf {
        PathBuf::new().join(self.base_dir)
    } 
}

impl<'a> IOStore for Store<'a> {
    fn get_current_field(&self) -> Result<Option<String>> {
        let pathbuf = self.get_basedir_pathbuf().join("fields/_CURRENT");
        let current_field = fs::read_to_string(pathbuf)?;

        Ok(if current_field.is_empty() { None } else { Some(current_field) })
    }

    fn get_fields(&self) -> Vec<String> {
        Vec::new()
    }

    fn create_field(&self, field: &str) -> Result<()> {
        let field_path = self.get_basedir_pathbuf().join("fields").join(field);
        fs::create_dir_all(field_path.join("paths"))?;
        fs::write(field_path.join("HEAD"), "main")?;
        fs::File::create(field_path.join("paths").join("main"))?;

        Ok(())
    }

    fn set_current_field(&self, field: &str) -> Result<()> {
        let file_path = self.get_basedir_pathbuf().join("fields").join("_CURRENT");
        fs::write(file_path, field)?;
        Ok(())
    }

    fn field_exists(&self, field: &str) -> bool {
      self.get_basedir_pathbuf().join("fields/").join(field).exists()  
    }

    fn get_paths(&self, field: &str) -> Vec<String> {
        Vec::new()
    }

    fn create_path(&self, field: &str, path: &str) -> Result<()> {
        Err(From::from("create_path: UNIMPLEMENTED"))
    }

    fn set_current_path(&self, field: &str, path: &str) -> Result<()> {
        Err(From::from("set_current_path: UNIMPLEMENTED"))
    }
    fn get_current_path(&self, field: &str) -> Result<Option<String>> {
        Err(From::from("get_current_path: UNIMPLEMENTED"))
    }

    fn path_exists(&self, field: &str, path: &str) -> bool {
      self.get_basedir_pathbuf()
        .join("fields")
        .join(field)
        .join("paths")
        .join(path)
        .exists()  
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store() {
        let base_dir = "tmp/ztln_store1";
        let path = Path::new(base_dir);
        let _store = Store::init(base_dir).unwrap();
        assert!(Store::init(base_dir).is_err());
        assert!(path.join("fields").is_dir());
        assert!(path.join("meta").is_dir());
        assert!(path.join("notes").is_dir());
        assert!(path.join("fields/_CURRENT").is_file());
        assert!(path.join("index").is_file());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_create_path() {
        let base_dir = "tmp/ztln_store2";
        let store = Store::init(base_dir).unwrap();
        let path = Path::new(base_dir);
        assert!(!path.join("fields").join("fieldA").exists());
        store.create_field("fieldA").unwrap();
        assert!(path.join("fields").join("fieldA").is_dir());
        assert!(path.join("fields").join("fieldA").join("HEAD").is_file());
        assert!(path.join("fields").join("fieldA").join("paths").is_dir());
        assert!(path.join("fields").join("fieldA").join("paths").join("main").is_file());

        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_set_current_field() {
        let base_dir = "tmp/ztln_store3";
        let mut store = Store::init(base_dir).unwrap();
        let path = Path::new(base_dir);
        store.create_field("fieldA").unwrap();
        assert!(store.set_current_field("fieldA").is_ok());
        assert_eq!("fieldA", store.get_current_field().unwrap().unwrap());
    }
}