use super::*;

impl WorldLayout {
    pub fn command_storage_file(&self, namespace: &str) -> PathBuf {
        self.data_dir()
            .join(namespace)
            .join("command_storage.dat")
    }

    pub fn save_command_storage(&self, namespace: &str, tag: &Tag) -> std::io::Result<()> {
        validate_saved_data_namespace(namespace)?;
        fs::create_dir_all(self.data_dir().join(namespace))?;
        let mut bytes = Vec::new();
        let tag = tag_with_data_version(tag);
        write_named_tag(&mut bytes, "", &tag)?;
        durable_write_with_backup(&self.command_storage_file(namespace), None, &bytes)
    }

    pub fn load_command_storage(&self, namespace: &str) -> std::io::Result<Tag> {
        validate_saved_data_namespace(namespace)?;
        let bytes = fs::read(self.command_storage_file(namespace))?;
        let (_name, tag) = read_named_tag(&mut bytes.as_slice())?;
        checked_saved_tag(&format!("{namespace}:command_storage"), tag)
    }
}

fn validate_saved_data_namespace(namespace: &str) -> std::io::Result<()> {
    let valid = !namespace.is_empty()
        && namespace
            .bytes()
            .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid saved data namespace",
        ))
    }
}
