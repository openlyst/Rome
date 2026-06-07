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
