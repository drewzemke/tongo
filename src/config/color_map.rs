use anyhow::{anyhow, bail, Result};
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
    panel: HashMap<String, String>,

    #[serde(default)]
    tab: HashMap<String, String>,

    #[serde(default)]
    popup: HashMap<String, String>,

    #[serde(default)]
    palette: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, strum_macros::EnumIter)]
pub enum ColorKey {
    // general ui
    FgPrimary,
    FgSecondary,
    SelectionBg,
    SelectionFg,
    IndicatorSuccess,
    IndicatorError,
    IndicatorInfo,
    AppName,

    // panel
    PanelActiveBg,
    PanelActiveBorder,
    PanelInactiveBg,
    PanelInactiveBorder,
    PanelActiveInputBorder,

    // popup
    PopupBg,
    PopupBorder,

    // tab
    TabActive,
    TabInactive,

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
}

/// A helper struct for parsing the color map.
struct ColorKeyMapping {
    section: &'static str,
    mappings: &'static [(&'static str, ColorKey)],
}

impl ColorKeyMapping {
    const UI: Self = Self {
        section: "ui",
        mappings: &[
            ("fg-primary", ColorKey::FgPrimary),
            ("fg-secondary", ColorKey::FgSecondary),
            ("selection-bg", ColorKey::SelectionBg),
            ("selection-fg", ColorKey::SelectionFg),
            ("indicator-success", ColorKey::IndicatorSuccess),
            ("indicator-error", ColorKey::IndicatorError),
            ("indicator-info", ColorKey::IndicatorInfo),
            ("app-name", ColorKey::AppName),
        ],
    };

    const PANEL: Self = Self {
        section: "panel",
        mappings: &[
            ("active-bg", ColorKey::PanelActiveBg),
            ("active-border", ColorKey::PanelActiveBorder),
            ("inactive-bg", ColorKey::PanelInactiveBg),
            ("inactive-border", ColorKey::PanelInactiveBorder),
            ("active-input-border", ColorKey::PanelActiveInputBorder),
        ],
    };

    const TAB: Self = Self {
        section: "tab",
        mappings: &[
            ("active", ColorKey::TabActive),
            ("inactive", ColorKey::TabInactive),
        ],
    };

    const POPUP: Self = Self {
        section: "popup",
        mappings: &[("border", ColorKey::PopupBorder), ("bg", ColorKey::PopupBg)],
    };

    const DATA: Self = Self {
        section: "data",
        mappings: &[
            ("boolean", ColorKey::Boolean),
            ("date", ColorKey::Date),
            ("key", ColorKey::Key),
            ("number", ColorKey::Number),
            ("object-id", ColorKey::ObjectId),
            ("string", ColorKey::String),
            ("punctuation", ColorKey::Punctuation),
            ("mongo-operator", ColorKey::MongoOperator),
        ],
    };

    const DOCUMENTS: Self = Self {
        section: "documents",
        mappings: &[
            ("note", ColorKey::DocumentsNote),
            ("search", ColorKey::DocumentsSearch),
        ],
    };
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
                ColorKey::FgPrimary => Color::White,
                ColorKey::FgSecondary => Color::Gray,
                ColorKey::SelectionBg => Color::White,
                ColorKey::SelectionFg => Color::Black,
                ColorKey::IndicatorSuccess => Color::Green,
                ColorKey::IndicatorError => Color::Red,
                ColorKey::IndicatorInfo => Color::Blue,
                ColorKey::AppName => Color::Magenta,

                // panel
                ColorKey::PanelActiveBg => Color::Reset,
                ColorKey::PanelActiveBorder => Color::Green,
                ColorKey::PanelInactiveBg => Color::Reset,
                ColorKey::PanelInactiveBorder => Color::White,
                ColorKey::PanelActiveInputBorder => Color::Yellow,

                // popup
                ColorKey::PopupBorder => Color::Blue,
                ColorKey::PopupBg => Color::Reset,

                // tab
                ColorKey::TabActive => Color::Green,
                ColorKey::TabInactive => Color::Gray,

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

        // create the palette
        let mut palette = HashMap::default();
        for (key_str, color_str) in &map.palette {
            if !key_str
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                bail!("Invalid palette key \"{key_str}\". (Keys may consist of alphanumeric characters or '_' or '-')")
            }
            let color = Color::from_str(color_str)
                .map_err(|_| anyhow!("Color not recognized: \"{color_str}\""))?;
            palette.insert(key_str.clone(), color);
        }

        // process each section
        color_map.process_section(&ColorKeyMapping::UI, &map.ui, &palette)?;
        color_map.process_section(&ColorKeyMapping::PANEL, &map.panel, &palette)?;
        color_map.process_section(&ColorKeyMapping::TAB, &map.tab, &palette)?;
        color_map.process_section(&ColorKeyMapping::POPUP, &map.popup, &palette)?;
        color_map.process_section(&ColorKeyMapping::DATA, &map.data, &palette)?;
        color_map.process_section(&ColorKeyMapping::DOCUMENTS, &map.documents, &palette)?;

        Ok(color_map)
    }
}

impl ColorMap {
    /// Converts a color string to a `ratatui::style::Color`, first by checking
    /// for a match in the palette and then using `Color::try_from`
    fn resolve_color(
        color_str: &str,
        palette: &HashMap<String, Color>,
    ) -> Result<Color, anyhow::Error> {
        palette.get(color_str).map_or_else(
            || {
                Color::from_str(color_str)
                    .map_err(|_| anyhow!("Color not recognized: \"{color_str}\""))
            },
            |color| Ok(*color),
        )
    }

    /// Processes the configured colors in a section of the configuration,
    /// updating the internal color map as necessary
    fn process_section(
        &mut self,
        mapping: &ColorKeyMapping,
        section_map: &HashMap<String, String>,
        palette: &HashMap<String, Color>,
    ) -> Result<()> {
        for (key_str, color_str) in section_map {
            let key = mapping
                .mappings
                .iter()
                .find(|(k, _)| *k == key_str)
                .map(|(_, color_key)| color_key)
                .ok_or_else(|| {
                    anyhow!(
                        "Theme key in \"{}\" section not recognized: \"{}\"",
                        mapping.section,
                        key_str
                    )
                })?;

            let color = Self::resolve_color(color_str, palette)?;
            self.map.insert(*key, color);
        }
        Ok(())
    }

    /// Gets the `Color` associated with a `ColorKey`
    ///
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
