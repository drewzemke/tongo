use anyhow::{anyhow, bail};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use strum::IntoEnumIterator;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RawColorMap {
    #[serde(default)]
    data: HashMap<String, String>,

    #[serde(default)]
    documents: HashMap<String, String>,

    #[serde(default)]
    ui: HashMap<String, String>,

    #[serde(default)]
    input: HashMap<String, String>,

    #[serde(default)]
    palette: HashMap<String, String>,
}

#[derive(Debug, Hash, PartialEq, Eq, strum_macros::EnumIter)]
pub enum ColorKey {
    // general ui
    Fg,
    FocusedPanelBg,
    FocusedPanelBorder,
    SelectionBg,
    SelectionFg,
    UnfocusedPanelBg,
    UnfocusedPanelBorder,

    // data
    Boolean,
    Date,
    Key,
    Number,
    ObjectId,
    String,
    Punctuation,
    MongoOperator,

    // documents
    DocumentsNote,
    DocumentsSearch,

    // input
    InputValid,
    InputInvalid,
    InputBorderActive,
}

#[derive(Debug)]
pub struct ColorMap {
    map: HashMap<ColorKey, Color>,
}

impl Default for ColorMap {
    fn default() -> Self {
        let mut map = HashMap::default();

        for key in ColorKey::iter() {
            #[expect(clippy::match_same_arms)]
            let color = match key {
                // general ui
                ColorKey::Fg => Color::White,
                ColorKey::FocusedPanelBg => Color::Reset,
                ColorKey::FocusedPanelBorder => Color::Green,
                ColorKey::SelectionBg => Color::White,
                ColorKey::SelectionFg => Color::Black,
                ColorKey::UnfocusedPanelBg => Color::Reset,
                ColorKey::UnfocusedPanelBorder => Color::White,

                //data
                ColorKey::Boolean => Color::Cyan,
                ColorKey::Date => Color::Magenta,
                ColorKey::Key => Color::White,
                ColorKey::Number => Color::Yellow,
                ColorKey::ObjectId => Color::White,
                ColorKey::String => Color::Green,
                ColorKey::Punctuation => Color::Gray,
                ColorKey::MongoOperator => Color::Magenta,

                // documents
                ColorKey::DocumentsNote => Color::Gray,
                ColorKey::DocumentsSearch => Color::Cyan,

                // input
                ColorKey::InputValid => Color::Green,
                ColorKey::InputInvalid => Color::Red,
                ColorKey::InputBorderActive => Color::Yellow,
            };

            map.insert(key, color);
        }

        Self { map }
    }
}

impl TryFrom<RawColorMap> for ColorMap {
    type Error = anyhow::Error;

    fn try_from(map: RawColorMap) -> Result<Self, Self::Error> {
        // create the palette
        let mut palette: HashMap<String, Color> = HashMap::default();
        for (key_str, color_str) in &map.palette {
            if key_str
                .chars()
                .any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
            {
                bail!("Invalid palette key \"{key_str}\". (Keys may consist of alphanumeric characters or '_' or '-')")
            }

            let color = Color::from_str(color_str)
                .map_err(|_| anyhow!("Color not recognized: \"{color_str}\""))?;

            palette.insert(key_str.clone(), color);
        }

        // create the actual color map
        let mut color_map = Self::default();
        for (key_str, color_str) in &map.ui {
            let key = match key_str as &str {
                "fg" => ColorKey::Fg,
                "focused_panel_bg" => ColorKey::FocusedPanelBg,
                "focused_panel_border" => ColorKey::FocusedPanelBorder,
                "selection_bg" => ColorKey::SelectionBg,
                "selection_fg" => ColorKey::SelectionFg,
                "unfocused_panel_bg" => ColorKey::UnfocusedPanelBg,
                "unfocused_panel_border" => ColorKey::UnfocusedPanelBorder,
                _ => bail!(format!("Theme key not recognized: \"{key_str}\"")),
            };

            // check if `color_str` refers to the palette
            let color = if let Some(color) = palette.get(color_str) {
                *color
            } else {
                Color::from_str(color_str)
                    .map_err(|_| anyhow!("Color not recognized: \"{color_str}\""))?
            };

            color_map.map.insert(key, color);
        }

        for (key_str, color_str) in &map.data {
            let key = match key_str as &str {
                "boolean" => ColorKey::Boolean,
                "date" => ColorKey::Date,
                "key" => ColorKey::Key,
                "number" => ColorKey::Number,
                "object_id" => ColorKey::ObjectId,
                "string" => ColorKey::String,
                "punctuation" => ColorKey::Punctuation,
                "mongo-operator" => ColorKey::MongoOperator,
                _ => bail!(format!("Theme key not recognized: \"{key_str}\"")),
            };

            // check if `color_str` refers to the palette
            let color = if let Some(color) = palette.get(color_str) {
                *color
            } else {
                Color::from_str(color_str)
                    .map_err(|_| anyhow!("Color not recognized: \"{color_str}\""))?
            };

            color_map.map.insert(key, color);
        }

        for (key_str, color_str) in &map.documents {
            let key = match key_str as &str {
                "note" => ColorKey::DocumentsNote,
                "search" => ColorKey::DocumentsSearch,
                _ => bail!(format!("Theme key not recognized: \"{key_str}\"")),
            };

            // check if `color_str` refers to the palette
            let color = if let Some(color) = palette.get(color_str) {
                *color
            } else {
                Color::from_str(color_str)
                    .map_err(|_| anyhow!("Color not recognized: \"{color_str}\""))?
            };

            color_map.map.insert(key, color);
        }

        for (key_str, color_str) in &map.input {
            let key = match key_str as &str {
                "indicator-valid" => ColorKey::InputValid,
                "indicator-invalid" => ColorKey::InputInvalid,
                "border-active" => ColorKey::InputBorderActive,
                _ => bail!(format!("Theme key not recognized: \"{key_str}\"")),
            };

            // check if `color_str` refers to the palette
            let color = if let Some(color) = palette.get(color_str) {
                *color
            } else {
                Color::from_str(color_str)
                    .map_err(|_| anyhow!("Color not recognized: \"{color_str}\""))?
            };

            color_map.map.insert(key, color);
        }

        Ok(color_map)
    }
}

impl ColorMap {
    /// # Panics
    /// If asked to get the color for a color key that hasn't been set (which shouldn't happen)
    #[must_use]
    pub fn get(&self, key: &ColorKey) -> Color {
        *self
            .map
            .get(key)
            .expect("Color map should have colors defined for all keys")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_color_map() {
        let color_map =
            ColorMap::try_from(RawColorMap::default()).expect("should be able to create color map");

        assert_eq!(color_map.get(&ColorKey::Key), Color::White);
    }

    #[test]
    fn color_map_with_ansi() {
        let mut raw_map = RawColorMap::default();
        raw_map
            .data
            .insert("boolean".to_string(), "yellow".to_string());
        raw_map
            .data
            .insert("string".to_string(), "cyan".to_string());

        let color_map = ColorMap::try_from(raw_map).expect("should be able to create color map");

        // check overridden values
        assert_eq!(color_map.get(&ColorKey::Boolean), Color::Yellow);
        assert_eq!(color_map.get(&ColorKey::String), Color::Cyan);

        // check default values remain unchanged
        assert_eq!(color_map.get(&ColorKey::Key), Color::White);
    }

    #[test]
    fn color_map_with_rgb() {
        let mut raw_map = RawColorMap::default();
        raw_map
            .data
            .insert("boolean".to_string(), "#FF0100".to_string());

        let color_map = ColorMap::try_from(raw_map).expect("should be able to create color map");

        // check overridden values
        assert_eq!(color_map.get(&ColorKey::Boolean), Color::Rgb(255, 1, 0));

        // check default values remain unchanged
        assert_eq!(color_map.get(&ColorKey::Key), Color::White);
    }

    #[test]
    fn invalid_color_map() {
        let mut raw_map = RawColorMap::default();
        raw_map
            .data
            .insert("invalid_key".to_string(), "white".to_string());

        assert!(ColorMap::try_from(raw_map).is_err());

        let mut raw_map = RawColorMap::default();
        raw_map
            .data
            .insert("boolean".to_string(), "invalid_color".to_string());

        assert!(ColorMap::try_from(raw_map).is_err());
    }

    #[test]
    fn color_map_with_palette() {
        let mut raw_map = RawColorMap::default();
        raw_map
            .palette
            .insert("lavender".to_string(), "#eeccff".to_string());
        raw_map
            .data
            .insert("boolean".to_string(), "lavender".to_string());

        let color_map = ColorMap::try_from(raw_map).expect("should be able to create color map");

        // check overridden values
        assert_eq!(color_map.get(&ColorKey::Boolean), Color::Rgb(238, 204, 255));
    }

    #[test]
    fn invalid_palette_keys() {
        let mut raw_map = RawColorMap::default();
        raw_map
            .palette
            .insert("#something".to_string(), "white".to_string());

        assert!(ColorMap::try_from(raw_map).is_err());

        let mut raw_map = RawColorMap::default();
        raw_map
            .palette
            .insert("rgb( )".to_string(), "white".to_string());

        assert!(ColorMap::try_from(raw_map).is_err());
    }
}
