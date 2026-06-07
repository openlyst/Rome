use rand::Rng;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::models::{BulkSystemRoms, Config, SearchSelection};

pub static USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.107 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.164 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:90.0) Gecko/20100101 Firefox/90.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.107 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36",
];

lazy_static::lazy_static! {
    pub static ref CONSOLES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Atari 7800", "Atari7800");
        m.insert("Nintendo NES", "NES");
        m.insert("Super Nintendo", "SNES");
        m.insert("Nintendo 64", "N64");
        m.insert("Wii", "Wii");
        m.insert("Wii Ware", "WiiWare");
        m.insert("Gamecube", "GameCube");
        m.insert("Master System", "SMS");
        m.insert("Mega Drive", "Genesis");
        m.insert("Saturn", "Saturn");
        m.insert("Dreamcast", "Dreamcast");
        m.insert("Playstation", "PS1");
        m.insert("Playstation 2", "PS2");
        m.insert("Playstation 3", "PS3");
        m.insert("Xbox", "Xbox");
        m.insert("Game Boy", "GB");
        m.insert("Game Boy Color", "GBC");
        m.insert("Game Boy Advanced", "GBA");
        m.insert("Nintendo DS", "DS");
        m.insert("Playstation Portable", "PSP");
        m
    };

    pub static ref COUNTRIES: HashMap<i32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(1, "Argentina");
        m.insert(2, "Asia");
        m.insert(3, "Australia");
        m.insert(35, "Austria");
        m.insert(31, "Belgium");
        m.insert(4, "Brazil");
        m.insert(5, "Canada");
        m.insert(6, "China");
        m.insert(38, "Croatia");
        m.insert(7, "Denmark");
        m.insert(8, "Europe");
        m.insert(9, "Finland");
        m.insert(10, "France");
        m.insert(11, "Germany");
        m.insert(12, "Greece");
        m.insert(13, "Hong Kong");
        m.insert(27, "India");
        m.insert(33, "Ireland");
        m.insert(34, "Israel");
        m.insert(14, "Italy");
        m.insert(15, "Japan");
        m.insert(16, "Korea");
        m.insert(30, "Latin America");
        m.insert(17, "Mexico");
        m.insert(18, "Netherlands");
        m.insert(40, "New Zealand");
        m.insert(19, "Norway");
        m.insert(28, "Poland");
        m.insert(29, "Portugal");
        m.insert(20, "Russia");
        m.insert(32, "Scandinavia");
        m.insert(37, "South Africa");
        m.insert(21, "Spain");
        m.insert(22, "Sweden");
        m.insert(36, "Switzerland");
        m.insert(23, "Taiwan");
        m.insert(39, "Turkey");
        m.insert(41, "UAE");
        m.insert(24, "United Kingdom");
        m.insert(25, "USA");
        m.insert(26, "World");
        m
    };
}

pub struct VimmsLairHelper {
    pub roms_directory: String,
    pub search_version: String,
    pub countries: Vec<i32>,
    pub all_countries: bool,
}

impl VimmsLairHelper {
    pub const VIMMS_LAIR_BASE_URL: &'static str = "https://vimm.net";
    pub const VIMMS_LAIR_DL_BASE_URL: &'static str = "https://dl2.vimm.net";
    pub const DIRNAMES: &[&str] = &[
        "#", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
        "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    ];

    pub fn new() -> Self {
        Self {
            roms_directory: String::from("ROMS"),
            search_version: String::from("new"),
            countries: vec![8, 14, 26],
            all_countries: false,
        }
    }

    pub fn print_welcome(&self) {
        println!(r#"
        _   _ _                          _           _     ______                    _                 _
        | | | (_)                        | |         (_)    |  _  \                  | |               | |
        | | | |_ _ __ ___  _ __ ___  ___ | |     __ _ _ _ __| | | |_____      ___ __ | | ___   __ _  __| | ___ _ __
        | | | | | '_ ` _ \| '_ ` _ \/ __|| |    / _` | | '__| | | / _ \ \ /\ / / '_ \| |/ _ \ / _` |/ _` |/ _ \ '__|
        \ \_/ / | | | | | | | | | | \__ \| |___| (_| | | |  | |/ / (_) \ V  V /| | | | | (_) | (_| | (_| |  __/ |
        \___/|_|_| |_| |_|_| |_| |_|___/\_____/\__,_|_|_|  |___/ \___/ \_/\_/ |_| |_|_|\___/ \__,_|\__,_|\___|_|
            "#);
        println!("Welcome to the Vimm's Lair Download Script!");
        println!("Please use responsibly, I am not liable for any damages or legal issues caused by using this script");
    }

    pub fn create_directory_structure(&self, config: &Config, path: &str) {
        if config.all {
            self.create_all_dirs(path);
        } else if !config.selections.is_empty() {
            self.create_dirs(path, &config.selections);
        }
    }

    pub fn selection_to_uri(&self, selection: &str) -> String {
        CONSOLES.get(selection).unwrap_or(&"").to_string()
    }

    pub fn print_console_list(&self) {
        let console_list: Vec<&str> = CONSOLES.keys().cloned().collect();
        let num_consoles = console_list.len();
        let half = (num_consoles + 1) / 2;

        for i in 0..half {
            let left = format!("{:2} ==> {}", i, console_list[i]);
            let right = if i + half < num_consoles {
                format!("{:2} ==> {}", i + half, console_list[i + half])
            } else {
                String::new()
            };
            println!("{:<30} {:<30}", left, right);
        }
    }

    pub fn get_selection_from_num(&self, selection: usize) -> String {
        let keys: Vec<&str> = CONSOLES.keys().cloned().collect();
        keys.get(selection).unwrap_or(&"").to_string()
    }

    pub fn get_random_ua(&self) -> String {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..USER_AGENTS.len());
        USER_AGENTS[idx].to_string()
    }

    pub fn get_search_url(&self, search_selection: &SearchSelection) -> String {
        let mut url = if search_selection.system != "general" {
            let console_uri_name = self.selection_to_uri(&search_selection.system);
            format!(
                "{}/vault/?p=list&action=filters&system={}&q={}",
                Self::VIMMS_LAIR_BASE_URL,
                console_uri_name,
                search_selection.query
            )
        } else {
            format!(
                "{}/vault/?p=list&action=filters&q={}",
                Self::VIMMS_LAIR_BASE_URL,
                search_selection.query
            )
        };
        url = self.add_url_filters(&url);
        url
    }

    pub fn generate_path_to_bulk_roms(
        &self,
        roms: &mut Vec<BulkSystemRoms>,
        home_dir: &str,
    ) {
        for system in roms.iter_mut() {
            for section in system.sections.iter_mut() {
                let folder_upper = section.section.chars().last().unwrap_or(' ').to_string().to_uppercase();
                let folder = if section.section.contains("number") {
                    "#"
                } else {
                    folder_upper.as_str()
                };
                section.path = Path::new(home_dir)
                    .join(&self.roms_directory)
                    .join(&system.system_name)
                    .join(folder)
                    .to_string_lossy()
                    .to_string();
            }
        }
    }

    pub fn add_url_filters(&self, url: &str) -> String {
        let mut result = url.to_string();
        if !self.search_version.is_empty() {
            result.push_str(&format!("&version={}", self.search_version));
        }
        if !self.countries.is_empty() {
            for country_id in &self.countries {
                if COUNTRIES.contains_key(country_id) {
                    result.push_str(&format!("&countries%5B%5D={}", country_id));
                }
            }
        }
        if self.all_countries {
            result.push_str("&countries_all=1");
        }
        result
    }

    fn create_alpha_num_structure(&self, path: &str, system: &str) {
        for dirname in Self::DIRNAMES {
            let dir_path = Path::new(path)
                .join(&self.roms_directory)
                .join(system)
                .join(dirname);
            if let Err(e) = fs::create_dir_all(&dir_path) {
                println!("Failed Creating AlphaNum Structure");
                println!("{}", e);
            }
        }
    }

    fn create_rom_home_dir(&self, path: &str) {
        let dir_path = Path::new(path).join(&self.roms_directory);
        if let Err(e) = fs::create_dir_all(&dir_path) {
            println!("Failed Creating Home Directory");
            println!("{}", e);
        }
    }

    fn create_rom_system_dir(&self, path: &str, system: &str) {
        let dir_path = Path::new(path).join(&self.roms_directory).join(system);
        if let Err(e) = fs::create_dir_all(&dir_path) {
            println!("Failed Creating ROM System Directory");
            println!("{}", e);
        }
    }

    fn is_home_dir_created(&self, path: &str) -> bool {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name == self.roms_directory {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_system_dir_created(&self, path: &str, system: &str) -> bool {
        let roms_path = Path::new(path).join(&self.roms_directory);
        if let Ok(entries) = fs::read_dir(&roms_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name == system {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn create_all_dirs(&self, path: &str) {
        let count = CONSOLES.len();
        let selections: Vec<usize> = (0..count).collect();
        self.create_dirs(path, &selections);
    }

    fn create_dirs(&self, path: &str, user_selections: &[usize]) {
        if !self.is_home_dir_created(path) {
            self.create_rom_home_dir(path);
        }
        for &selection in user_selections {
            let console_name = self.get_selection_from_num(selection);
            if !self.is_system_dir_created(path, &console_name) {
                self.create_rom_system_dir(path, &console_name);
                self.create_alpha_num_structure(path, &console_name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SectionOfRoms;

    #[test]
    fn test_vimms_lair_helper_new() {
        let helper = VimmsLairHelper::new();
        assert_eq!(helper.roms_directory, "ROMS");
        assert_eq!(helper.search_version, "new");
        assert_eq!(helper.countries, vec![8, 14, 26]);
        assert!(!helper.all_countries);
    }

    #[test]
    fn test_selection_to_uri() {
        let helper = VimmsLairHelper::new();
        assert_eq!(helper.selection_to_uri("Nintendo NES"), "NES");
        assert_eq!(helper.selection_to_uri("Super Nintendo"), "SNES");
        assert_eq!(helper.selection_to_uri("Nintendo 64"), "N64");
        assert_eq!(helper.selection_to_uri("Playstation"), "PS1");
        assert_eq!(helper.selection_to_uri("Nonexistent Console"), "");
    }

    #[test]
    fn test_get_selection_from_num() {
        let helper = VimmsLairHelper::new();
        let keys: Vec<&str> = CONSOLES.keys().cloned().collect();
        assert_eq!(helper.get_selection_from_num(0), keys[0]);
        assert_eq!(helper.get_selection_from_num(1), keys[1]);
        // Out of bounds should return empty string
        assert_eq!(helper.get_selection_from_num(999), "");
    }

    #[test]
    fn test_get_random_ua() {
        let helper = VimmsLairHelper::new();
        let ua1 = helper.get_random_ua();
        let ua2 = helper.get_random_ua();
        // Both should be from the known list
        assert!(USER_AGENTS.contains(&ua1.as_str()));
        assert!(USER_AGENTS.contains(&ua2.as_str()));
    }

    #[test]
    fn test_get_search_url_system() {
        let helper = VimmsLairHelper::new();
        let ss = SearchSelection {
            system: "Nintendo NES".to_string(),
            query: "contra".to_string(),
        };
        let url = helper.get_search_url(&ss);
        assert!(url.contains("https://vimm.net/vault/?p=list&action=filters&system=NES&q=contra"));
        assert!(url.contains("version=new"));
    }

    #[test]
    fn test_get_search_url_general() {
        let helper = VimmsLairHelper::new();
        let ss = SearchSelection {
            system: "general".to_string(),
            query: "metroid".to_string(),
        };
        let url = helper.get_search_url(&ss);
        assert!(url.contains("https://vimm.net/vault/?p=list&action=filters&q=metroid"));
        assert!(!url.contains("system="));
        assert!(url.contains("version=new"));
    }

    #[test]
    fn test_add_url_filters() {
        let helper = VimmsLairHelper::new();
        let base = "https://vimm.net/vault/?p=list&action=filters&system=NES&q=mario";
        let result = helper.add_url_filters(base);
        assert!(result.contains("version=new"));
        assert!(result.contains("countries%5B%5D=8"));
        assert!(result.contains("countries%5B%5D=14"));
        assert!(result.contains("countries%5B%5D=26"));
    }

    #[test]
    fn test_add_url_filters_all_countries() {
        let mut helper = VimmsLairHelper::new();
        helper.all_countries = true;
        helper.countries.clear();
        let base = "https://vimm.net/vault/?p=list&action=filters&q=mario";
        let result = helper.add_url_filters(base);
        assert!(result.contains("countries_all=1"));
    }

    #[test]
    fn test_consoles_map_populated() {
        assert!(!CONSOLES.is_empty());
        assert!(CONSOLES.contains_key("Nintendo NES"));
        assert!(CONSOLES.contains_key("Playstation 2"));
        assert_eq!(CONSOLES.get("Nintendo NES"), Some(&"NES"));
    }

    #[test]
    fn test_countries_map_populated() {
        assert!(!COUNTRIES.is_empty());
        assert_eq!(COUNTRIES.get(&25), Some(&"USA"));
        assert_eq!(COUNTRIES.get(&8), Some(&"Europe"));
    }

    #[test]
    fn test_generate_path_to_bulk_roms() {
        let helper = VimmsLairHelper::new();
        let mut roms = vec![BulkSystemRoms::new(
            vec![
                SectionOfRoms::new("section=number".to_string(), vec![]),
                SectionOfRoms::new("section=A".to_string(), vec![]),
            ],
            "NES".to_string(),
            "Nintendo NES".to_string(),
        )];
        helper.generate_path_to_bulk_roms(&mut roms, "/tmp/test");

        assert!(roms[0].sections[0].path.contains("#"));
        assert!(roms[0].sections[1].path.contains("A"));
    }
}
