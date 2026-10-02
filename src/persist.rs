use std::path::{Path, PathBuf};

use crate::model::Settings;

fn settings_path_from(xdg_config_home: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
    let dir = match xdg_config_home {
        Some(x) if x.is_absolute() => x,
        _ => match home {
            Some(h) => h.join(".config"),
            None => return PathBuf::from("settings.json"),
        },
    };
    dir.join("rclipthing").join("settings.json")
}

pub fn settings_path() -> PathBuf {
    settings_path_from(
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

pub fn load_settings(path: &Path) -> Settings {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(path: &Path, settings: &Settings) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Theme;

    #[test]
    fn test_load_missing_returns_default() {
        let s = load_settings(Path::new("/tmp/definitely_missing_settings.json"));
        assert_eq!(s.folders, vec!["./images".to_string()]);
    }

    #[test]
    fn test_roundtrip() {
        let mut s = Settings::default();
        s.theme = Theme::Light;
        s.folders = vec!["/a".to_string(), "/b".to_string()];
        s.thumb_size = 150.0;
        s.top_n = 25;
        let tmp = Path::new("/tmp/rclipthing_settings_test.json");
        save_settings(tmp, &s);
        let loaded = load_settings(tmp);
        let _ = std::fs::remove_file(tmp);
        assert_eq!(loaded.theme, Theme::Light);
        assert_eq!(loaded.folders, vec!["/a".to_string(), "/b".to_string()]);
        assert_eq!(loaded.thumb_size, 150.0);
        assert_eq!(loaded.top_n, 25);
    }

    #[test]
    fn test_old_settings_json_maps_to_defaults() {
        let tmp = Path::new("/tmp/rclipthing_old_settings.json");
        std::fs::write(
            tmp,
            r#"{"theme":"dark","chat_width":500.0,"chat_font_size":16.0,"ui_scale":1.3,"notes_folder":"./"}"#,
        )
        .unwrap();
        let loaded = load_settings(tmp);
        let _ = std::fs::remove_file(tmp);
        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.ui_scale, 1.3);
        assert_eq!(loaded.folders, vec!["./images".to_string()]);
    }

    #[test]
    fn test_settings_path_from() {
        let xdg = |s: &str| Some(PathBuf::from(s));
        let home = |s: &str| Some(PathBuf::from(s));

        assert_eq!(
            settings_path_from(xdg("/custom"), home("/home/u")),
            PathBuf::from("/custom/rclipthing/settings.json")
        );
        assert_eq!(
            settings_path_from(xdg("relative"), home("/home/u")),
            PathBuf::from("/home/u/.config/rclipthing/settings.json"),
            "non absolute XDG_CONFIG_HOME must be ignored"
        );
        assert_eq!(
            settings_path_from(None, home("/home/u")),
            PathBuf::from("/home/u/.config/rclipthing/settings.json")
        );
        assert_eq!(
            settings_path_from(None, None),
            PathBuf::from("settings.json"),
            "no HOME: fall back to cwd"
        );
    }

    #[test]
    fn test_save_creates_parent_dirs() {
        let tmp = Path::new("/tmp/rclipthing_nested/cfg/settings.json");
        let _ = std::fs::remove_dir_all("/tmp/rclipthing_nested");
        save_settings(tmp, &Settings::default());
        assert!(tmp.exists());
        let _ = std::fs::remove_dir_all("/tmp/rclipthing_nested");
    }
}
