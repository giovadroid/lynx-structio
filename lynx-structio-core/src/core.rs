use std::path::PathBuf;
use crate::{MonitorResult};
use crate::file_monitor::GLOBAL_FILE_MONITOR;
use crate::traits::{ FileStructTrait, Updatable};

pub fn save<Instance, Serializable>(instance: &Instance) -> MonitorResult<()>
    where Instance: Updatable<Serializable>,
          Serializable: serde::ser::Serialize + serde::de::DeserializeOwned
{
    let path = Instance::path();
    if !path.exists() && !path.parent().unwrap().exists() {
        std::fs::create_dir_all(path.parent().unwrap())?;
    }
    std::fs::write(&path, serde_yaml::to_string(&instance.content())?)?;
    log::debug!("Saved to {:?}", path);
    Ok(())
}

pub fn load<Structure, Deserializable>() -> MonitorResult<Structure>
    where Structure: Updatable<Deserializable> + Default + FileStructTrait,
          Deserializable: serde::de::DeserializeOwned + serde::ser::Serialize
{

    let instance = Structure::default();

    if !Structure::path().exists() {
        log::debug!("Path {:?} does not exist, creating", Structure::path());
        instance.save()?;
    }

    reload(&instance)?;
    Ok(instance)
}

pub fn reload<Structure, Deserializable>(instance: &Structure) -> MonitorResult<()>
    where Structure: Updatable<Deserializable> ,
          Deserializable: serde::de::DeserializeOwned + serde::ser::Serialize
{
    let path: PathBuf = Structure::path();

    let reader = std::fs::File::open(&path)?;
    let data_type = serde_yaml::from_reader::<_, Deserializable>(reader)?;
    instance.update(data_type);
    log::info!("Data updated from {:?}", path);
    Ok(())
}

pub fn monitor<Structure, Deserializable>(instance: &Structure) -> MonitorResult<()>
    where Structure: Updatable<Deserializable> + Clone + FileStructTrait + Send + Sync + 'static,
          Deserializable: serde::de::DeserializeOwned + serde::ser::Serialize
{
    let callback_patterns = instance.clone();
    GLOBAL_FILE_MONITOR.register_file(Structure::path().to_str().unwrap().to_owned(), move || {
        if let Err(e) = callback_patterns.reload() {
            log::error!("Error updating patterns: {}", e);
        }
    })?;
    Ok(())
}