use chrono::{DateTime, Utc};

#[derive(PartialEq, Debug, Clone)]
pub struct Image {
    id: i64,
    path: String,
    updated_on: DateTime<Utc>,
}

impl Image {
    pub fn new(id: i64, path: String, updated_on: DateTime<Utc>) -> Self {
        Self {
            id,
            path,
            updated_on,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn path(&self) -> &str {
        self.path.as_ref()
    }

    pub fn updated_on(&self) -> DateTime<Utc> {
        self.updated_on
    }
}
