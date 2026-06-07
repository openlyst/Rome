pub struct Rom {
    pub name: String,
    pub page_url: String,
    pub download_url: String,
    pub system: String,
    pub region: String,
    pub version: String,
}

impl Rom {
    pub fn new(
        name: String,
        page_url: String,
        download_url: String,
        system: String,
        region: String,
        version: String,
    ) -> Self {
        Self {
            name,
            page_url,
            download_url,
            system,
            region,
            version,
        }
    }

    pub fn new_basic(name: String, page_url: String, download_url: String) -> Self {
        Self {
            name,
            page_url,
            download_url,
            system: String::new(),
            region: String::new(),
            version: String::new(),
        }
    }
}

pub struct SectionOfRoms {
    pub section: String,
    pub roms: Vec<Rom>,
    pub path: String,
}

impl SectionOfRoms {
    pub fn new(section: String, roms: Vec<Rom>) -> Self {
        Self {
            section,
            roms,
            path: String::new(),
        }
    }
}

pub struct SearchSelection {
    pub system: String,
    pub query: String,
}

impl Default for SearchSelection {
    fn default() -> Self {
        Self {
            system: String::new(),
            query: String::new(),
        }
    }
}

pub struct BulkSystemRoms {
    pub system: String,
    pub system_name: String,
    pub sections: Vec<SectionOfRoms>,
}

impl BulkSystemRoms {
    pub fn new(sections: Vec<SectionOfRoms>, system: String, system_name: String) -> Self {
        Self {
            system,
            system_name,
            sections,
        }
    }
}

pub struct Search {
    pub search_selections: SearchSelection,
    pub general: bool,
}

impl Search {
    pub fn new(search_selections: SearchSelection) -> Self {
        Self {
            search_selections,
            general: false,
        }
    }
}

pub struct Config {
    pub selections: Vec<usize>,
    pub all: bool,
    pub extract: bool,
    pub search_mode: bool,
    pub bulk_mode: bool,
    pub query_selection: SearchSelection,
    pub query: Search,
}

impl Default for Config {
    fn default() -> Self {
        let query_selection = SearchSelection::default();
        let query = Search::new(SearchSelection {
            system: String::new(),
            query: String::new(),
        });
        Self {
            selections: Vec::new(),
            all: false,
            extract: false,
            search_mode: false,
            bulk_mode: false,
            query_selection,
            query,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_new() {
        let rom = Rom::new(
            "Super Mario Bros".to_string(),
            "https://vimm.net/vault/1234".to_string(),
            "https://dl2.vimm.net/download?mediaId=1234".to_string(),
            "NES".to_string(),
            "USA".to_string(),
            "1.0".to_string(),
        );
        assert_eq!(rom.name, "Super Mario Bros");
        assert_eq!(rom.system, "NES");
        assert_eq!(rom.region, "USA");
        assert_eq!(rom.version, "1.0");
    }

    #[test]
    fn test_rom_new_basic() {
        let rom = Rom::new_basic(
            "Zelda".to_string(),
            "https://vimm.net/vault/5678".to_string(),
            "https://dl2.vimm.net/download?mediaId=5678".to_string(),
        );
        assert_eq!(rom.name, "Zelda");
        assert!(rom.system.is_empty());
        assert!(rom.region.is_empty());
        assert!(rom.version.is_empty());
    }

    #[test]
    fn test_section_of_roms_new() {
        let roms = vec![
            Rom::new_basic("Game1".to_string(), "/1".to_string(), "/dl1".to_string()),
        ];
        let section = SectionOfRoms::new("A".to_string(), roms);
        assert_eq!(section.section, "A");
        assert_eq!(section.roms.len(), 1);
        assert!(section.path.is_empty());
    }

    #[test]
    fn test_search_selection_default() {
        let ss = SearchSelection::default();
        assert!(ss.system.is_empty());
        assert!(ss.query.is_empty());
    }

    #[test]
    fn test_bulk_system_roms_new() {
        let sections = vec![SectionOfRoms::new("A".to_string(), vec![])];
        let bsr = BulkSystemRoms::new(sections, "NES".to_string(), "Nintendo NES".to_string());
        assert_eq!(bsr.system, "NES");
        assert_eq!(bsr.system_name, "Nintendo NES");
        assert_eq!(bsr.sections.len(), 1);
    }

    #[test]
    fn test_search_new() {
        let ss = SearchSelection {
            system: "NES".to_string(),
            query: "mario".to_string(),
        };
        let search = Search::new(ss);
        assert_eq!(search.search_selections.system, "NES");
        assert_eq!(search.search_selections.query, "mario");
        assert!(!search.general);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.selections.is_empty());
        assert!(!config.all);
        assert!(!config.extract);
        assert!(!config.search_mode);
        assert!(!config.bulk_mode);
        assert!(config.query_selection.system.is_empty());
        assert!(config.query_selection.query.is_empty());
    }
}
