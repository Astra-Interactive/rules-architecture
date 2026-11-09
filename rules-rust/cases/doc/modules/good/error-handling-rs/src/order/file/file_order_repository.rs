use crate::order::model::{Order, OrderError, OrderId};
use crate::order::util::parse_order;
use std::fs;
use std::path::PathBuf;

pub(crate) struct FileOrderRepository {
    folder: PathBuf,
}

impl FileOrderRepository {
    pub(crate) fn new(folder: PathBuf) -> Self {
        Self { folder }
    }

    fn path_of(&self, id: OrderId) -> PathBuf {
        self.folder.join(id.0.to_string())
    }

    fn load_order(&self, id: OrderId) -> Result<Order, OrderError> {
        let raw = fs::read_to_string(self.path_of(id)).map_err(|error| OrderError::Load {
            id,
            source: error.into(),
        })?;
        parse_order(&raw)
    }
}
