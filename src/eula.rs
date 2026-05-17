use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Eula {
    pub agreed: bool,
}

impl Eula {
    pub fn load_or_create(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            fs::write(
                path,
                "# By changing the setting below to TRUE you are indicating your agreement to the Minecraft EULA.\n\
                 # https://aka.ms/MinecraftEULA\n\
                 eula=false\n",
            )
            .map_err(|err| format!("Failed to create '{}': {err}", path.display()))?;
        }

        let text = fs::read_to_string(path)
            .map_err(|err| format!("Failed to read '{}': {err}", path.display()))?;
        Ok(Self {
            agreed: parse_eula_agreement(&text),
        })
    }
}

fn parse_eula_agreement(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        !line.starts_with('#') && line.eq_ignore_ascii_case("eula=true")
    })
}

#[cfg(test)]
mod tests {
    use super::parse_eula_agreement;

    #[test]
    fn parses_eula_true_case_insensitively() {
        assert!(parse_eula_agreement("EULA=TRUE\n"));
    }

    #[test]
    fn ignores_commented_agreement() {
        assert!(!parse_eula_agreement("# eula=true\neula=false\n"));
    }
}
