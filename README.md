
 This crate provides derive macro to automatically implement the FileStructTrait trait for a struct
 
 The FileStructTrait trait provides the save, load, and reload functions

 Optionally, the struct can be watched for changes in the file

 Requires the serde Serialize and Deserialize traits


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

// Can use FileStruct instead of FileWatch to only save and load the struct without monitoring the file
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
