use crate::model::Settings;

pub fn load_settings(path: &str) -> Settings {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(path: &str, settings: &Settings) {
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
        let s = load_settings("/tmp/definitely_missing_settings.json");
        assert_eq!(s.folders, vec!["./images".to_string()]);
    }

    #[test]
    fn test_roundtrip() {
        let mut s = Settings::default();
        s.theme = Theme::Light;
        s.folders = vec!["/a".to_string(), "/b".to_string()];
        s.thumb_size = 150.0;
        s.top_n = 25;
        let tmp = "/tmp/rclipthing_settings_test.json";
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
        let tmp = "/tmp/rclipthing_old_settings.json";
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
}