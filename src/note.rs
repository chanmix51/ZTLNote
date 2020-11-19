use uuid::Uuid;
use crate::error::{ZtlnError, Result};

pub struct NoteMetaData {
    note_id: Uuid,
    parent_id: Uuid,
    references: Vec<Uuid>,
}

impl NoteMetaData {
    pub fn parse_meta_file(filename: &str, content: &str) -> Result<Self> {
        let note_id = Uuid::parse_str(filename)?;
        let mut lines = content.lines();
        let parent_id = lines.next().ok_or(ZtlnError::CannotParseNote)?;
        let parent_id = Uuid::parse_str(parent_id)?;
        let mut references = Vec::new();
        for reference in lines {
            references.push(Uuid::parse_str(reference)?);
        }
        Ok(Self { note_id, parent_id, references })
    }
}