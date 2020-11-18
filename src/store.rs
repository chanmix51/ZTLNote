use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{ZtlnError, Result};

pub trait IOStore {
    fn get_fields(&self) -> Vec<String>;
    fn create_field(&self, name: &str) -> Result<()>;
    fn set_current_field(&self, field: String) -> Result<()>;
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
            return Err(From::from(ZtlnError::new(format!("Given directory '{}' already exists.", base_dir))));
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
            return Err(From::from(ZtlnError::new(format!("Given path '{}' is not a directory.", base_dir))));
        }

        if !(
            path.join("meta").is_dir()
            && path.join("notes").is_dir()
            && path.join("index").is_file()
            && path.join("fields").is_dir()
            && path.join("fields/__CURRENT").is_file()
            ) {
            return Err(From::from(ZtlnError::new(format!("Invalid ztln structure in dir '{}'.", base_dir))))
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

        Ok(if current_field.len() == 0 { None } else { Some(current_field) })
    }

    fn get_fields(&self) -> Vec<String> {
        Vec::new()
    }

    fn create_field(&self, name: &str) -> Result<()> {
        Ok(())
    }

    fn set_current_field(&self, field: String) -> Result<()> {
        Ok(())
    }

    fn field_exists(&self, field: &str) -> bool {
      self.get_basedir_pathbuf().join("fields/").join(field).exists()  
    }

    fn get_paths(&self, field: &str) -> Vec<String> {
        Vec::new()
    }

    fn create_path(&self, field: &str, path: &str) -> Result<()> {
        Ok(())
    }

    fn set_current_path(&self, field: &str, path: &str) -> Result<()> {
        Ok(())
    }
    fn get_current_path(&self, field: &str) -> Result<Option<String>> {
        Ok(None)
    }

    fn path_exists(&self, field: &str, path: &str) -> bool {
      self.get_basedir_pathbuf().join("fields").join(field).join("paths").join(path).exists()  
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store() {
        let base_dir = "tmp/ztln_test";
        let path = Path::new(base_dir);
        let store = Store::init(base_dir).unwrap();
        assert!(Store::init(base_dir).is_err());
        assert!(path.join("fields").is_dir());
        assert!(path.join("meta").is_dir());
        assert!(path.join("notes").is_dir());
        assert!(path.join("fields/_CURRENT").is_file());
        assert!(path.join("index").is_file());

        fs::remove_dir_all(path).unwrap();
    }
}