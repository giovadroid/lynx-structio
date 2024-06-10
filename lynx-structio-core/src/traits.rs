
use std::path::PathBuf;
use serde::{de, ser};
use crate::MonitorResult;


pub trait Updatable<Inner>
where Inner: ser::Serialize + de::DeserializeOwned
{
    fn update(&self, new_data: Inner);
    fn path() -> PathBuf;
    fn content(&self) -> Inner;
}

pub trait FileStructTrait {
    fn save(&self) -> MonitorResult<()>;
    fn load() -> MonitorResult<Self> where Self: Sized;
    fn reload(&self) -> MonitorResult<()>;
}