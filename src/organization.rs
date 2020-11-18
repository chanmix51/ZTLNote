use crate::store::{Store, IOStore};
use crate::error::{ZtlnError, Result};

#[derive(Debug)]
pub struct Organization<'a> {
    current_field: Option<String>,
    store: Store<'a>,
}

impl<'a> Organization<'a> {
    pub fn new(store: Store<'a>) -> Self {
        Self {
            current_field: None,
            store
        }
    }

    pub fn get_current_field(&mut self) -> Option<String> {
        if self.current_field.is_none() {
            let field = self.store
                .get_current_field()
                .unwrap_or_else(|e| self.manage_store_error::<_>(e));
            if field == None {
                return None;
            }
            self.current_field = field;
        }

        (self.current_field).clone()
    }

    pub fn set_current_field(&mut self, field: &str) -> Result<()> {
        if !self.store.field_exists(field) {
            Err(From::from(ZtlnError::FieldDoesNotExist(field.to_string())))
        } else {
            self.store.set_current_field(field)
                .unwrap_or_else(|e| self.manage_store_error::<_>(e));
            self.current_field = Some(field.to_string());
            Ok(())
        }
    }

    pub fn create_field(&mut self, field: &str) -> Result<()> {
        if self.store.field_exists(field) {
            Err(From::from(ZtlnError::FieldAlreadyExists(field.to_string())))
        } else {
            self.store.create_field(field)
                .unwrap_or_else(|e| self.manage_store_error::<_>(e));
            Ok(())
        }
    }

    pub fn get_current_path(&self, field: &str) -> Result<Option<String>> {
        if self.store.field_exists(field) {
            self.store.get_current_path(field)
        } else {
            Err(From::from(ZtlnError::FieldDoesNotExist(field.to_string())))
        }
    }

    fn manage_store_error<T>(&self, err: Box<dyn std::error::Error>) -> T {
        eprintln!("IO ERROR: {:?}", err);
        panic!("PANIC!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_organization() {
        let base_dir = "tmp/ztln_orga1";
        let store = Store::init(base_dir);
        assert!(store.is_ok());
        let mut orga = Organization::new(store.unwrap());
        assert_eq!(None, orga.get_current_field());
        
        let store = Store::init(base_dir);
        assert!(store.is_err());

        std::fs::remove_dir_all(std::path::Path::new(base_dir)).unwrap();
    }

    #[test]
    fn get_current_field() {
        let base_dir = "tmp/ztln_orga2";
        let mut orga = Organization::new( Store::init(base_dir).unwrap());

        assert_eq!("NONE", orga.get_current_field().unwrap_or_else(|| "NONE".to_string()));
        orga.create_field("field1").unwrap();
        assert_eq!("NONE", orga.get_current_field().unwrap_or_else(|| "NONE".to_string()));
        orga.set_current_field("field1").unwrap();
        assert_eq!("field1", orga.get_current_field().unwrap_or_else(|| "NONE".to_string()));
        orga.create_field("field2").unwrap();
        assert_eq!("field1", orga.get_current_field().unwrap_or_else(|| "NONE".to_string()));
        orga.set_current_field("field2").unwrap();
        assert_eq!("field2", orga.get_current_field().unwrap_or_else(|| "NONE".to_string()));
        assert!(orga.set_current_field("field3").is_err());

        std::fs::remove_dir_all(std::path::Path::new(base_dir)).unwrap();
    }
}