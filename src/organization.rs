use crate::store::{Store, IOStore};
use crate::error::{ZtlnError, Result};
use uuid::Uuid;
use regex::{Regex, CaptureMatches};

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

    pub fn set_current_path(&mut self, topic: Option<&str>, path: &str) -> Result<()> {
        let topic = self.unwrap_or_default_topic(topic)?;
        if self.store.path_exists(&topic, path) {
            self.store.set_current_path(&topic, path)
                .unwrap_or_else(|e| self.manage_store_error::<_>(e));
                Ok(())
        } else {
            Err(From::from(ZtlnError::PathDoesNotExist(topic, path.to_string())))
        }
    }

    pub fn create_path(&mut self, topic: Option<&str>, new_path: &str, starting_path: Option<&str>) -> Result<()> {
        let topic = self.unwrap_or_default_topic(topic)?;
        let starting_path = match starting_path {
            Some(path) => path.to_string(),
            None => 
                match self.get_current_path(&topic)? {
                    Some(path) => path,
                    None => return Err(From::from(ZtlnError::Default("No default path".to_string()))),
            }
        };

        if self.store.path_exists(&topic, new_path) {
            return Err(From::from(ZtlnError::PathAlreadyExists(topic, new_path.to_string())))
        }
        if !self.store.path_exists(&topic, &starting_path) {
            return Err(From::from(ZtlnError::PathDoesNotExist(topic, starting_path)));
        }
        let uuid = self.store.get_path(&topic, &starting_path)
            .unwrap_or_else(|e| self.manage_store_error::<_>(e));
        self.store.write_path(&topic, new_path, uuid)?;
        Ok(())
    }

    pub fn get_paths_list(&mut self, topic: Option<&str>) -> Result<(String, Vec<String>)> {
        let topic = self.unwrap_or_default_topic(topic)?;
        let paths = self.store.get_paths(&topic)
                .unwrap_or_else(|e| self.manage_store_error::<_>(e));
        Ok((topic, paths))
    }

    pub fn add_note(&mut self, filename: &str, topic: Option<&str>, path: Option<&str>) -> Result<NoteCreationReport> {
        if let Some(f)= topic {
            self.set_current_topic(f)?;
        } else if self.get_current_topic().is_none() {
            return Err(From::from(ZtlnError::Default("No default topic".to_string())));
        }
        let topic = self.get_current_topic().unwrap();

        // Path management is a bit complex since this may be the first note to be created in a path.
        // In this case, there is no existing path hence one must be created and set as default.
        // 1 is a path provided?
        if let Some(new_path) = path {
            // 1.1 does it exist?
            if self.store.path_exists(&topic, new_path) {
                self.set_current_path(Some(&topic), new_path)?
            // 1.2 if not, if a default path exist, create a new path branching from it
            } else if let Some(curr) = self.get_current_path(&topic)? {
                let uuid = self.store.get_path(&topic, &curr)?;
                self.store.write_path(&topic, &new_path, uuid)?;
                self.set_current_path(Some(&topic), new_path)?;
            // 1.3 otherwise create a new branch from scratch
            } else {
                self.store.set_current_path(&topic, new_path)
                    .unwrap_or_else(|e| self.manage_store_error(e));
            }
        // 2 no path provided, if no default path exist, create abitrary "main"
        } else if self.get_current_path(&topic)?.is_none() {
            self.store.set_current_path(&topic, "main")
                .unwrap_or_else(|e| self.manage_store_error(e));
        }
        let path = self.get_current_path(&topic)?.unwrap();
        let meta = self.store.add_note(&topic, &path, filename)?;
        
        Ok(NoteCreationReport { note_id: meta.note_id, parent_id: meta.parent_id, topic, path })
    }

    fn solve_location(&mut self, expr: &str) -> Result<Option<Uuid>> {
        lazy_static! {
            static ref RELATIVE_LOC: Regex = Regex::new(r"^(?:(?P<topic>\w+)/)?(?P<path>\w+)(?::-(?P<modifier>\d+))?$").unwrap();
            static ref ABSOLUTE_LOC: Regex = Regex::new(r"^(?P<subuuid>[[:xdigit:]]{8})(?:(?:-[[:xdigit:]]{4}){3}-[[:xdigit:]]{12})?$").unwrap();
        }
        if RELATIVE_LOC.is_match(expr) {
            let mut captures = RELATIVE_LOC.captures_iter(expr);
            self.solve_relative(&mut captures)
        } else if ABSOLUTE_LOC.is_match(expr) {
            let mut captures = ABSOLUTE_LOC.captures_iter(expr);
            self.solve_absolute(&mut captures)
        } else {
            Err(From::from(ZtlnError::Default(format!("Invalid location '{}'.", expr))))
        }
    }

    fn solve_absolute(&mut self, captures: &mut CaptureMatches) -> Result<Option<Uuid>> {
        let cap = captures.next().unwrap();
        let subuuid = cap.name("subuuid").unwrap().as_str().to_string();
        let some_uuid = self.store.search_short_uuid(&subuuid)?
            .map(|meta| meta.note_id);

        Ok(some_uuid)
    }
    fn solve_relative(&mut self, captures: &mut CaptureMatches) -> Result<Option<Uuid>> {
        let cap = captures.next().unwrap();
        let path = cap.name("path").unwrap().as_str().to_string();
        let topic = if cap.name("topic").is_none() { 
            match self.get_current_topic() {
                Some(t) => t,
                None => return Err(From::from(ZtlnError::Default("No default topic and no topic specified.".to_string()))),
            }
        } else { cap.name("topic").unwrap().as_str().to_string() };
        let mut some_uuid = if self.store.path_exists(&topic, &path) {
            self.store.get_path(&topic, &path).ok()
        } else { None };
        let mut modifier = if cap.name("modifier").is_none() {
            0_usize
        } else { 
            str::parse::<usize>(cap.name("modifier").unwrap().as_str())?
        };

        while modifier > 0 && some_uuid.is_some() {
            some_uuid = self.store
                .get_note_metadata(some_uuid.unwrap())?
                .map(|meta| meta.parent_id)
                .flatten();
            modifier -= 1;
        }

        Ok(some_uuid)
    }

    fn manage_store_error<T>(&self, err: Box<dyn std::error::Error>) -> T {
        eprintln!("IO ERROR: {:?}", err);
        panic!("Crashing the application…");
    }

    fn unwrap_or_default_topic(&mut self, topic: Option<&str>) -> Result<String> {
        let topic = if let Some(t) = topic {
            t.to_string()
        } else {
            self.get_current_topic()
                .ok_or_else(|| ZtlnError::Default("No topic given.".to_string()))?
        };

        Ok(topic)
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
        std::fs::write(filename, "This is test 3 content").unwrap();
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
        let topic = "topic3";
        let path = "path1";
        orga.create_topic(topic).unwrap();
        let res6 = orga.add_note(filename, Some(topic), Some(path));
        assert!(res6.is_ok());

        std::fs::remove_dir_all(std::path::Path::new(base_dir)).unwrap();
    }

    #[test]
    fn create_path() {
        let base_dir = "tmp/ztln_orga4";
        let filename = "tmp/test4";
        let topic = "topic1";
        let mut orga = Organization::new( Store::init(base_dir).unwrap());
        let res = orga.create_path(None, "whatever", None);
        assert!(res.is_err());
        orga.create_topic(topic).unwrap();
        let res = orga.create_path(Some(topic), "whatever", None);
        assert!(res.is_err());
        std::fs::write(filename, "This is test 4 content").unwrap();
        let report1 = orga.add_note(filename, Some(topic), None).unwrap();
        let res1 = orga.create_path(Some(topic), "path2", None);
        assert!(res1.is_ok());
        assert_eq!(2, orga.get_paths_list(Some(topic)).unwrap().1.len());
        let report2 = orga.add_note(filename, Some(topic), Some("path2")).unwrap();
        assert_eq!(report1.note_id, report2.parent_id.unwrap());
        let res1 = orga.create_path(Some("wrong"),  "whatever", None);
        assert!(res1.is_err());

        std::fs::remove_dir_all(std::path::Path::new(base_dir)).unwrap();
    }

    #[test]
    fn note_location_ok() {
        let expressions:&[&str] = &[
            "HEAD",
            "main",
            "whatever",
            "topic1/main",
            "topic1/HEAD",
            "topic1/whatever",
            "whatever/main",
            "HEAD:-0",
            "main:-2",
            "topic1/main:-3",
            "topic1/HEAD:-10",
            "topic1/whatever:-1",
            "whatever/main:-0",
            "44a0f45f",
            "44a0f45f-22b6-4675-a277-e196d8881ca8"
        ];

        let base_dir = "tmp/ztln_orga5";
        let filename = "tmp/test5";
        let topic = "topic1";
        std::fs::write(filename, "This is test 5 content").unwrap();
        let mut orga = Organization::new( Store::init(base_dir).unwrap());
        orga.create_topic(topic).unwrap();
        orga.set_current_topic(topic).unwrap();
        orga.add_note(filename, None, None).unwrap();

        for expr in expressions {
            println!("Testing location '{}' is good…", expr);
            assert!(orga.solve_location(expr).is_ok());
        }
    }

    #[test]
    fn note_location_wrong() {
        let expressions:&[&str] = &[
            "",
            "tata:toto",
            "tata:+1",
            "44a0f45f-22b6",
            "tata/toto/tete",
        ];

        let base_dir = "tmp/ztln_orga6";
        let filename = "tmp/test6";
        let topic = "topic1";
        std::fs::write(filename, "This is test 6 content").unwrap();
        let mut orga = Organization::new( Store::init(base_dir).unwrap());
        orga.create_topic(topic).unwrap();
        orga.set_current_topic(topic).unwrap();
        orga.add_note(filename, None, None).unwrap();

        for expr in expressions {
            println!("Testing location '{}' is wrong…", expr);
            assert!(orga.solve_location(expr).is_err());
        }
    }

}