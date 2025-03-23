use anyhow::bail;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RawColorMap {
    documents: HashMap<String, String>,
}

#[derive(Debug, Hash, PartialEq, Eq, strum_macros::EnumIter)]
pub enum ColorKey {
    DocumentsKey,
    DocumentsObjectId,
    DocumentsString,
    DocumentsBoolean,
    DocumentsNumber,
    DocumentsDate,
}

#[derive(Debug)]
pub struct ColorMap {
    map: HashMap<ColorKey, Color>,
}

impl Default for ColorMap {
    fn default() -> Self {
        let mut map = HashMap::default();

        for key in ColorKey::iter() {
            let color = match key {
                ColorKey::DocumentsKey | ColorKey::DocumentsObjectId => Color::White,
                ColorKey::DocumentsString => Color::Green,
                ColorKey::DocumentsBoolean => Color::Cyan,
                ColorKey::DocumentsNumber => Color::Yellow,
                ColorKey::DocumentsDate => Color::Magenta,
            };

            map.insert(key, color);
        }

        Self { map }
    }
}

impl TryFrom<RawColorMap> for ColorMap {
    type Error = anyhow::Error;

    fn try_from(map: RawColorMap) -> Result<Self, Self::Error> {
        let mut color_map = Self::default();

        for (key_str, color_str) in &map.documents {
            let key = match key_str as &str {
                "boolean" => ColorKey::DocumentsBoolean,
                "date" => ColorKey::DocumentsDate,
                "key" => ColorKey::DocumentsKey,
                "number" => ColorKey::DocumentsNumber,
                "object_id" => ColorKey::DocumentsObjectId,
                "string" => ColorKey::DocumentsString,
                _ => bail!(format!("Theme key not recognized: \"{key_str}\"")),
            };
            let color = match color_str as &str {
                "cyan" => Color::Cyan,
                "green" => Color::Green,
                "magenta" => Color::Magenta,
                "white" => Color::White,
                "yellow" => Color::Yellow,
                _ => bail!(format!("Color value not recognized: \"{color_str}\"")),
            };
            color_map.map.insert(key, color);
        }

        Ok(color_map)
    }
}

impl ColorMap {
    #[must_use]
    pub fn get(&self, key: &ColorKey) -> Option<&Color> {
        self.map.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_color_map() {
        let color_map =
            ColorMap::try_from(RawColorMap::default()).expect("should be able to create color map");

        assert_eq!(color_map.get(&ColorKey::DocumentsKey), Some(&Color::White));
    }

    #[test]
    fn test_raw_color_map_conversion() {
        let mut raw_map = RawColorMap::default();
        raw_map
            .documents
            .insert("boolean".to_string(), "yellow".to_string());
        raw_map
            .documents
            .insert("string".to_string(), "cyan".to_string());

        let color_map = ColorMap::try_from(raw_map).expect("should be able to create color map");

        // check overridden values
        assert_eq!(
            color_map.get(&ColorKey::DocumentsBoolean),
            Some(&Color::Yellow)
        );
        assert_eq!(
            color_map.get(&ColorKey::DocumentsString),
            Some(&Color::Cyan)
        );

        // check default values remain unchanged
        assert_eq!(color_map.get(&ColorKey::DocumentsKey), Some(&Color::White));
    }

    #[test]
    fn test_invalid_raw_color_map() {
        let mut raw_map = RawColorMap::default();
        raw_map
            .documents
            .insert("invalid_key".to_string(), "white".to_string());

        assert!(ColorMap::try_from(raw_map).is_err());

        let mut raw_map = RawColorMap::default();
        raw_map
            .documents
            .insert("boolean".to_string(), "invalid_color".to_string());

        assert!(ColorMap::try_from(raw_map).is_err());
    }
}
