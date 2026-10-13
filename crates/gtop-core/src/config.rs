use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub refresh_ms: u64,
    pub hide_kernel: bool,
    pub default_sort: String,
    pub show_full_cmdline: bool,
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            refresh_ms: 1000,
            hide_kernel: true,
            default_sort: "cpu".to_string(),
            show_full_cmdline: false,
            theme: "default".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(dirs) = directories::ProjectDirs::from("io", "gtop", "gtop") {
            let cfg_path = dirs.config_dir().join("config.toml");
            if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                if let Ok(cfg) = toml::from_str(&content) {
                    return cfg;
                }
            }
        }
        Config::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(dirs) = directories::ProjectDirs::from("io", "gtop", "gtop") {
            let cfg_dir = dirs.config_dir();
            std::fs::create_dir_all(cfg_dir)?;
            let content = toml::to_string_pretty(self)?;
            std::fs::write(cfg_dir.join("config.toml"), content)?;
        }
        Ok(())
    }
}
