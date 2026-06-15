use std::fs;
use std::path::Path;

const EULA_URL: &str = "https://aka.ms/MinecraftEULA";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Eula {
    agreed: bool,
}

impl Eula {
    pub fn load_or_create(path: &Path) -> Result<Self, String> {
        Self::load_or_create_with_ide(path, false)
    }

    pub fn load_or_create_with_ide(path: &Path, is_running_in_ide: bool) -> Result<Self, String> {
        Ok(Self {
            agreed: is_running_in_ide || read_file(path)?,
        })
    }

    pub fn has_agreed_to_eula(&self) -> bool {
        self.agreed
    }
}

fn read_file(path: &Path) -> Result<bool, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(parse_eula_agreement(&text)),
        Err(_) => {
            let _ = save_defaults(path);
            Ok(false)
        }
    }
}

fn save_defaults(path: &Path) -> Result<(), String> {
    fs::write(
        path,
        format!(
            "#By changing the setting below to TRUE you are indicating your agreement to our EULA ({}).\n\
             eula=false\n",
            EULA_URL
        ),
    )
    .map_err(|err| format!("Failed to save '{}': {err}", path.display()))
}

fn parse_eula_agreement(text: &str) -> bool {
    for line in text.lines() {
        let line = line.trim_start();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        let Some((key, value)) = split_property_line(line) else {
            continue;
        };
        if key == "eula" && value.eq_ignore_ascii_case("true") {
            return true;
        }
    }
    false
}

fn split_property_line(line: &str) -> Option<(&str, &str)> {
    let mut separator_index = None;
    for (index, ch) in line.char_indices() {
        if ch == '=' || ch == ':' || ch.is_whitespace() {
            separator_index = Some(index);
            break;
        }
    }

    let Some(index) = separator_index else {
        return Some((line.trim_end(), ""));
    };

    let key = line[..index].trim_end();
    let mut value_start = index + line[index..].chars().next()?.len_utf8();
    while let Some(ch) = line[value_start..].chars().next() {
        if ch.is_whitespace() {
            value_start += ch.len_utf8();
        } else {
            break;
        }
    }
    if matches!(line[value_start..].chars().next(), Some('=') | Some(':')) {
        value_start += 1;
        while let Some(ch) = line[value_start..].chars().next() {
            if ch.is_whitespace() {
                value_start += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    Some((key, line[value_start..].trim_end()))
}

#[cfg(test)]
mod tests {
    use super::{parse_eula_agreement, Eula};
    use std::fs;
    use std::path::PathBuf;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/Eula.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_eula_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("private final Path file;"));
        assert!(JAVA_SOURCE.contains("private final boolean agreed;"));
        assert!(JAVA_SOURCE.contains(
            "this.agreed = SharedConstants.IS_RUNNING_IN_IDE || this.readFile();"
        ));
        assert!(JAVA_SOURCE.contains("properties.load(input);"));
        assert!(JAVA_SOURCE
            .contains("Boolean.parseBoolean(properties.getProperty(\"eula\", \"false\"))"));
        assert!(JAVA_SOURCE.contains("public boolean hasAgreedToEULA()"));
        assert!(JAVA_SOURCE.contains("properties.setProperty(\"eula\", \"false\");"));
    }

    #[test]
    fn server_utility_eula_parses_true_case_insensitively() {
        assert!(parse_eula_agreement("eula=TRUE\n"));
        assert!(!parse_eula_agreement("EULA=TRUE\n"));
    }

    #[test]
    fn server_utility_eula_ignores_commented_agreement() {
        assert!(!parse_eula_agreement("# eula=true\neula=false\n"));
    }

    #[test]
    fn server_utility_eula_accepts_java_properties_separators() {
        assert!(parse_eula_agreement("eula = true\n"));
        assert!(parse_eula_agreement("eula:true\n"));
        assert!(parse_eula_agreement("eula true\n"));
        assert!(!parse_eula_agreement("eula = false\n"));
    }

    #[test]
    fn server_utility_eula_missing_file_writes_default_and_refuses() {
        let dir = temp_dir("missing-file");
        let path = dir.join("eula.txt");

        let eula = Eula::load_or_create(&path).unwrap_or_else(|err| panic!("{err}"));

        assert!(!eula.has_agreed_to_eula());
        let written = fs::read_to_string(&path).unwrap_or_else(|err| panic!("{err}"));
        assert!(written.contains("eula=false"));
        assert!(written.contains("https://aka.ms/MinecraftEULA"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn server_utility_eula_ide_mode_agrees_without_creating_file() {
        let dir = temp_dir("ide-mode");
        let path = dir.join("eula.txt");

        let eula = Eula::load_or_create_with_ide(&path, true)
            .unwrap_or_else(|err| panic!("{err}"));

        assert!(eula.has_agreed_to_eula());
        assert!(!path.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vibecraft-eula-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap_or_else(|err| panic!("{err}"));
        dir
    }
}
