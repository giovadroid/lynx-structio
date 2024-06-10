
 This crate provides a way to monitor files and execute a callback when a file is modified
 Also, it provides a way to save and load structs to and from files
 Requires the serde Serialize and Deserialize traits

 Also provides a proc macro to derive the Save and Load traits or to watch a file for changes


 ### Example
 ```rust
 use lynx_structio::prelude::*;
 use serde::{Serialize, Deserialize};
 use std::path::PathBuf;
 use std::sync::Arc;
 use parking_lot::RwLock;

 #[derive(Serialize, Deserialize, Default, Clone)]
 struct DataStruct {
    data: String,
    number: i32,
 }

 #[derive(FileWatch, Default, Clone)]
 struct FileDataStruct{
     data: Arc<RwLock<DataStruct>>,
 }

 impl Updatable<DataStruct> for FileDataStruct {
     fn update(&self, new_data: DataStruct) {
         self.data.write().data = new_data.data;
         self.data.write().number = new_data.number;
     }

     fn path() -> PathBuf {
         PathBuf::from("data.yaml")
     }

     fn content(&self) -> DataStruct {
         self.data.read().clone()
     }
 }

 #[tokio::main]
 async fn main() {
     let data = FileDataStruct::load().unwrap();

     data.reload().unwrap();
     data.save().unwrap()
 }
 ```
