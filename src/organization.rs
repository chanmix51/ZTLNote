use crate::store::{Store, IOStore};
use crate::error::{ZtlnError, Result};
use uuid::Uuid;

#[derive(Debug)]
pub struct NoteCreationReport {
    pub note_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub topic: String,
    pub path: String,
}

#[derive(Debug)]
pub struct Organization<'a> {
    current_topic: Option<String>,
    store: Store<'a>,
}

impl<'a> Organization<'a> {
    pub fn new(store: Store<'a>) -> Self {
        Self {
            current_topic: None,
            store
        }
    }

    pub fn get_current_topic(&mut self) -> Option<String> {
        if self.current_topic.is_none() {
            let topic = self.store
                .get_current_topic()
                .unwrap_or_else(|e| self.manage_store_error::<_>(e));
            if topic == None {
                return None;
            }
            self.current_topic = topic;
        }

        (self.current_topic).clone()
    }

    pub fn set_current_topic(&mut self, topic: &str) -> Result<()> {
        if !self.store.topic_exists(topic) {
            Err(From::from(ZtlnError::TopicDoesNotExist(topic.to_string())))
        } else {
            self.store.set_current_topic(topic)
                .unwrap_or_else(|e| self.manage_store_error::<_>(e));
            self.current_topic = Some(topic.to_string());
            Ok(())
        }
    }

    pub fn create_topic(&mut self, topic: &str) -> Result<()> {
        if self.store.topic_exists(topic) {
            Err(From::from(ZtlnError::TopicAlreadyExists(topic.to_string())))
        } else {
            self.store.create_topic(topic)
                .unwrap_or_else(|e| self.manage_store_error::<_>(e));
            if self.get_current_topic().is_none() {
                self.set_current_topic(topic)
                    .unwrap_or_else(|e| self.manage_store_error::<_>(e));
                self.current_topic = Some(topic.to_string());
            }
            Ok(())
        }
    }

    pub fn get_topics_list(&self) -> Vec<String> {
        self.store.get_topics()
                .unwrap_or_else(|e| self.manage_store_error::<_>(e))
    }

    pub fn get_current_path(&self, topic: &str) -> Result<Option<String>> {
        if self.store.topic_exists(topic) {
            self.store.get_current_path(topic)
        } else {
            Err(From::from(ZtlnError::TopicDoesNotExist(topic.to_string())))
        }
    }

    pub fn set_current_path(&self, topic: &str, path: &str) -> Result<()> {
        if self.store.path_exists(topic, path) {
            self.store.set_current_path(topic, path)
        } else {
            Err(From::from(ZtlnError::PathDoesNotExist(topic.to_string(), path.to_string())))
        }
    }

    pub fn create_path(&self, topic: &str, new_path: &str, starting_path: &str) -> Result<()> {
        if self.store.path_exists(topic, new_path) {
            return Err(From::from(ZtlnError::PathAlreadyExists(topic.to_string(), new_path.to_string())))
        }
        if !self.store.path_exists(topic, starting_path) {
            return Err(From::from(ZtlnError::PathDoesNotExist(topic.to_string(), starting_path.to_string())));
        }
        let uuid = self.store.get_path(topic, starting_path)
            .unwrap_or_else(|e| self.manage_store_error::<_>(e));
        self.store.write_path(topic, new_path, uuid)?;
        Ok(())
    }

    pub fn get_paths_list(&self, topic: &str) -> Vec<String> {
        self.store.get_paths(topic)
                .unwrap_or_else(|e| self.manage_store_error::<_>(e))
    }

    pub fn add_note(&mut self, filename: &str, topic: Option<&str>, path: Option<&str>) -> Result<NoteCreationReport> {
        if let Some(f)= topic {
            self.set_current_topic(f)?;
        } else if self.get_current_topic().is_none() {
            return Err(From::from(ZtlnError::Default("No default topic".to_string())));
        }
        let topic = self.get_current_topic().unwrap();

        if let Some(new_path) = path {
            if self.store.path_exists(&topic, new_path) {
                self.set_current_path(&topic, new_path)?
            } else if let Some(curr) = self.get_current_path(&topic)? {
                let uuid = self.store.get_path(&topic, &curr)?;
                self.store.write_path(&topic, &new_path, uuid)?;
                self.set_current_path(&topic, new_path)?;
            }
        } else if self.get_current_path(&topic)?.is_none() {
            self.store.set_current_path(&topic, "main")
                .unwrap_or_else(|e| self.manage_store_error(e));
        }
        let path = self.get_current_path(&topic)?.unwrap();
        let meta = self.store.add_note(&topic, &path, filename)?;
        
        Ok(NoteCreationReport { note_id: meta.note_id, parent_id: meta.parent_id, topic, path })
    }

    fn manage_store_error<T>(&self, err: Box<dyn std::error::Error>) -> T {
        eprintln!("IO ERROR: {:?}", err);
        panic!("Crashing the application…");
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
        assert_eq!(None, orga.get_current_topic());
        
        let store = Store::init(base_dir);
        assert!(store.is_err());

        std::fs::remove_dir_all(std::path::Path::new(base_dir)).unwrap();
    }

    #[test]
    fn get_current_topic() {
        let base_dir = "tmp/ztln_orga2";
        let mut orga = Organization::new( Store::init(base_dir).unwrap());

        assert_eq!("NONE", orga.get_current_topic().unwrap_or_else(|| "NONE".to_string()));
        orga.create_topic("topic1").unwrap();
        assert_eq!("topic1", orga.get_current_topic().unwrap_or_else(|| "NONE".to_string()));
        orga.set_current_topic("topic1").unwrap();
        assert_eq!("topic1", orga.get_current_topic().unwrap_or_else(|| "NONE".to_string()));
        orga.create_topic("topic2").unwrap();
        assert_eq!("topic1", orga.get_current_topic().unwrap_or_else(|| "NONE".to_string()));
        orga.set_current_topic("topic2").unwrap();
        assert_eq!("topic2", orga.get_current_topic().unwrap_or_else(|| "NONE".to_string()));
        assert!(orga.set_current_topic("topic3").is_err());

        std::fs::remove_dir_all(std::path::Path::new(base_dir)).unwrap();
    }

    #[test]
    fn add_note() {
        let base_dir = "tmp/ztln_orga3";
        let filename = "tmp/test3";
        let topic = "topic1";
        let mut orga = Organization::new( Store::init(base_dir).unwrap());
        orga.create_topic(topic).unwrap();
        std::fs::write("tmp/test3", "This is test 3 content").unwrap();
        let res1 = orga.add_note(filename, None, None).unwrap();
        assert!(res1.parent_id.is_none());
        assert_eq!(topic, res1.topic);
        assert_eq!("main", res1.path);
        assert_eq!("main", orga.get_current_path(topic).unwrap().unwrap());
        let res2 = orga.add_note(filename, None, None).unwrap();
        assert_eq!(Some(res1.note_id), res2.parent_id);
        let res3 = orga.add_note(filename, None, Some("path1")).unwrap();
        assert_eq!("path1", orga.get_current_path(topic).unwrap().unwrap());
        assert_eq!(Some(res2.note_id), res3.parent_id);
        assert!(orga.store.path_exists(topic, "path1"));
        let res4 = orga.add_note(filename, Some("wrong"), None);
        assert!(res4.is_err());
        let topic = "topic2";
        orga.create_topic(topic).unwrap();
        orga.set_current_topic(topic).unwrap();
        let res5 = orga.add_note(filename, None, None).unwrap();
        assert!(res5.parent_id.is_none());
        assert_eq!(topic, res5.topic);
        assert_eq!("main", res5.path);

        std::fs::remove_dir_all(std::path::Path::new(base_dir)).unwrap();
    }
}