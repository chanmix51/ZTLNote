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
                .unwrap_or_else(|e| self.manage_error::<_>(e));
            if field == None {
                return None;
            }
            self.current_field = field;
        }

        (self.current_field).clone()
    }

    pub fn get_current_path(&self, field: &str) -> Result<Option<String>> {
        if self.store.field_exists(field) {
            self.store.get_current_path(field)
        } else {
            Err(From::from(ZtlnError::new(format!("No such thought field '{}'.", field))))
        }
    }

    fn manage_error<T>(&self, err: Box<dyn std::error::Error>) -> T {
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
}