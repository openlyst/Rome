mod helpers;
mod models;

use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::io::{self, Write};
use std::path::Path;
use std::fs::{self, File};
use zip::ZipArchive;

use helpers::{VimmsLairHelper, CONSOLES};
use models::{BulkSystemRoms, Config, Rom, SearchSelection, SectionOfRoms};

fn get_rom_download_url(vimmslair_helper: &VimmsLairHelper, page_url: &str) -> String {
    let mut download_id = String::new();
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    match client.get(page_url).send() {
        Ok(response) => {
            if let Ok(text) = response.text() {
                let document = Html::parse_document(&text);
                let form_selector = Selector::parse("#dl_form").unwrap();
                if let Some(form) = document.select(&form_selector).next() {
                    let input_selector = Selector::parse("input[name=\"mediaId\"]").unwrap();
                    if let Some(input) = form.select(&input_selector).next() {
                        if let Some(value) = input.value().attr("value") {
                            download_id = value.to_string();
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed on getting ROM ID");
            println!("{}", e);
        }
    }
    format!("{}?mediaId={}", VimmsLairHelper::VIMMS_LAIR_DL_BASE_URL, download_id)
}

fn get_sub_section_letter_from_str(subsection: &str) -> String {
    if subsection.to_lowercase().contains("&section=number") {
        "number".to_string()
    } else {
        subsection.chars().last().unwrap_or(' ').to_string()
    }
}

fn get_section_of_roms(vimmslair_helper: &VimmsLairHelper, section: &str) -> Vec<Rom> {
    let mut roms: Vec<Rom> = Vec::new();
    let section_url = format!("{}/vault/{}", VimmsLairHelper::VIMMS_LAIR_BASE_URL, section);
    let section_url = vimmslair_helper.add_url_filters(&section_url);
    println!("Getting a list of roms for the section: {}", section);
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    match client.get(&section_url).send() {
        Ok(response) => {
            if let Ok(text) = response.text() {
                let document = Html::parse_document(&text);
                let table_selector = Selector::parse("table.rounded.centered.cellpadding1.hovertable.striped").unwrap();
                if let Some(table) = document.select(&table_selector).next() {
                    let row_selector = Selector::parse("tr").unwrap();
                    let td_selector = Selector::parse("td").unwrap();
                    let a_selector = Selector::parse("a").unwrap();
                    for row in table.select(&row_selector) {
                        let tds: Vec<_> = row.select(&td_selector).collect();
                        if let Some(first_td) = tds.first() {
                            if let Some(rom_a) = first_td.select(&a_selector).next() {
                                let name = rom_a.text().collect::<String>().trim().to_string();
                                let href = rom_a.value().attr("href").unwrap_or("").to_string();
                                let page_url = format!("{}{}", VimmsLairHelper::VIMMS_LAIR_BASE_URL, href);
                                let download_url = get_rom_download_url(vimmslair_helper, &page_url);
                                let rom = Rom::new_basic(name, page_url, download_url);
                                roms.push(rom);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed on getting section of roms: {}. The page may not contain any roms. Error: {}", section, e);
        }
    }
    roms
}

fn get_every_system_roms(vimmslair_helper: &VimmsLairHelper) -> Vec<BulkSystemRoms> {
    let mut all_roms: Vec<BulkSystemRoms> = Vec::new();
    let count = CONSOLES.len();
    for i in 0..count {
        let selection = vimmslair_helper.get_selection_from_num(i);
        let system_uri = vimmslair_helper.selection_to_uri(&selection);
        let system_roms = get_all_system_roms(vimmslair_helper, &system_uri, &selection);
        all_roms.push(system_roms);
    }
    all_roms
}

fn get_selected_system_bulk_roms(vimmslair_helper: &VimmsLairHelper, config: &Config) -> Vec<BulkSystemRoms> {
    let mut selected_bulk: Vec<BulkSystemRoms> = Vec::new();
    for &i in &config.selections {
        let selection = vimmslair_helper.get_selection_from_num(i);
        let system_uri = vimmslair_helper.selection_to_uri(&selection);
        let system_roms = get_all_system_roms(vimmslair_helper, &system_uri, &selection);
        selected_bulk.push(system_roms);
    }
    selected_bulk
}

fn get_all_system_roms(vimmslair_helper: &VimmsLairHelper, system: &str, system_name: &str) -> BulkSystemRoms {
    println!("Getting a list of roms for the {}", system_name);
    let mut section_roms: Vec<SectionOfRoms> = Vec::new();
    let sections = vec![
        "number", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J",
        "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U",
        "V", "W", "X", "Y", "Z",
    ];
    for sec in sections {
        let section_url = format!("?p=list&action=filters&system={}&section={}", system, sec);
        let roms = get_section_of_roms(vimmslair_helper, &section_url);
        let section = SectionOfRoms::new(section_url, roms);
        section_roms.push(section);
    }
    BulkSystemRoms::new(section_roms, system.to_string(), system_name.to_string())
}

fn download_file(vimmslair_helper: &VimmsLairHelper, download_url: &str, path: &str) -> String {
    let mut x = 0;
    let mut filename = String::new();
    let rom_id = download_url.split('/').last().unwrap_or("");
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    loop {
        let headers = {
            let mut h = reqwest::header::HeaderMap::new();
            h.insert(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"
                    .parse()
                    .unwrap(),
            );
            h.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
            h.insert("Connection", "keep-alive".parse().unwrap());
            h.insert("User-Agent", vimmslair_helper.get_random_ua().parse().unwrap());
            h.insert(
                "Referer",
                format!("{}/vault/{}", VimmsLairHelper::VIMMS_LAIR_BASE_URL, rom_id)
                    .parse()
                    .unwrap(),
            );
            h.insert("Sec-Fetch-Dest", "document".parse().unwrap());
            h.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
            h
        };
        match client.get(download_url).headers(headers.clone()).send() {
            Ok(response) => {
                if response.status().is_success() {
                    if let Some(content_disposition) = response.headers().get("Content-Disposition") {
                        if let Ok(cd_str) = content_disposition.to_str() {
                            let re = Regex::new(r#""([^"]*)""#).unwrap();
                            if let Some(caps) = re.captures(cd_str) {
                                if let Some(cap) = caps.get(1) {
                                    filename = cap.as_str().to_string();
                                }
                            }
                        }
                    }
                    let full_path = Path::new(path).join(&filename);
                    if let Ok(bytes) = response.bytes() {
                        if let Ok(mut file) = File::create(&full_path) {
                            use std::io::Write;
                            let _ = file.write_all(&bytes);
                        }
                    }
                    println!("Downloaded {}!", filename);
                    break;
                }
            }
            Err(e) => {
                if x == 4 {
                    println!("5 Requests made to {} and failed", download_url);
                    break;
                }
                x += 1;
                continue;
            }
        }
        if x == 4 {
            println!("5 Requests made to {} and failed", download_url);
            break;
        }
        x += 1;
        continue;
    }
    filename
}

fn get_search_selection(vimmslair_helper: &VimmsLairHelper, mut config: Config) -> Config {
    let mut search_selection = SearchSelection::default();
    println!("\nPlease select what system you want to search, or press Enter to do a general site wide search\n");
    vimmslair_helper.print_console_list();
    loop {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        if user_input.trim().is_empty() {
            search_selection.system = "general".to_string();
            config.query.search_selections = search_selection;
            break;
        }
        match user_input.trim().parse::<usize>() {
            Ok(num) => {
                let max = CONSOLES.len() - 1;
                if num <= max {
                    search_selection.system = vimmslair_helper.get_selection_from_num(num);
                    config.query.search_selections = search_selection;
                    break;
                } else {
                    println!("Not a selection");
                    println!("Please select a value from the list");
                }
            }
            Err(_) => {
                println!("Please select a value from the list");
                continue;
            }
        }
    }
    print!("Input what rom you want to search for: ");
    io::stdout().flush().unwrap();
    let mut query = String::new();
    io::stdin().read_line(&mut query).unwrap();
    config.query.search_selections.query = query.trim().to_string();
    config
}

fn get_search_section(vimmslair_helper: &VimmsLairHelper, search_selection: &SearchSelection) -> Vec<Rom> {
    let mut roms: Vec<Rom> = Vec::new();
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    match client.get(&vimmslair_helper.get_search_url(search_selection)).send() {
        Ok(response) => {
            if let Ok(text) = response.text() {
                let document = Html::parse_document(&text);
                let table_selector = Selector::parse("table.rounded.centered.cellpadding1.hovertable.striped").unwrap();
                if let Some(table) = document.select(&table_selector).next() {
                    let row_selector = Selector::parse("tr").unwrap();
                    let td_selector = Selector::parse("td").unwrap();
                    let a_selector = Selector::parse("a").unwrap();
                    let img_selector = Selector::parse("img").unwrap();
                    let b_selector = Selector::parse("b.redBorder").unwrap();
                    for row in table.select(&row_selector) {
                        if row.select(&Selector::parse("th").unwrap()).next().is_some() {
                            continue;
                        }
                        let row_tds: Vec<_> = row.select(&td_selector).collect();
                        let rom_name_td = row_tds.get(0);
                        let region_td = row_tds.get(1);
                        let version_td = row_tds.get(2);

                        if let Some(first_td) = rom_name_td {
                            if let Some(rom_a) = first_td.select(&a_selector).next() {
                                let mut name = rom_a.text().collect::<String>().trim().to_string();
                                if let Some(red_border) = first_td.select(&b_selector).next() {
                                    if let Some(title) = red_border.value().attr("title") {
                                        name.push(' ');
                                        name.push_str(title);
                                    }
                                }
                                let href = rom_a.value().attr("href").unwrap_or("").to_string();
                                let page_url = format!("{}{}", VimmsLairHelper::VIMMS_LAIR_BASE_URL, href);
                                let download_url = get_rom_download_url(vimmslair_helper, &page_url);

                                let region = if let Some(rtd) = region_td {
                                    let imgs: Vec<_> = rtd.select(&img_selector).collect();
                                    if !imgs.is_empty() {
                                        let mut region_parts = Vec::new();
                                        for (idx, img) in imgs.iter().enumerate() {
                                            let part = img.value().attr("title").unwrap_or("").to_string();
                                            region_parts.push(part);
                                            if idx != imgs.len() - 1 {
                                                region_parts.push("\\".to_string());
                                            }
                                        }
                                        region_parts.concat()
                                    } else {
                                        rtd.text().collect::<String>().trim().to_string()
                                    }
                                } else {
                                    "-".to_string()
                                };
                                let version = if let Some(vtd) = version_td {
                                    vtd.text().collect::<String>().trim().to_string()
                                } else {
                                    String::new()
                                };

                                let rom = Rom::new(
                                    name,
                                    page_url,
                                    download_url,
                                    search_selection.system.clone(),
                                    region,
                                    version,
                                );
                                roms.push(rom);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed on system search section: {}", e);
        }
    }
    roms
}

fn get_program_mode() -> Config {
    let mut config = Config::default();
    println!("\nWould you like to bulk download roms for systems or search for specific roms? (B/s)");
    println!("For bulk mode use 'b' and search mode use 's'");
    println!("Default is 'b'");
    loop {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        let trimmed = user_input.trim();
        if trimmed.is_empty() {
            config.bulk_mode = true;
            break;
        }
        if trimmed.to_lowercase() == "b" {
            config.bulk_mode = true;
            break;
        }
        if trimmed.to_lowercase() == "s" {
            config.search_mode = true;
            break;
        } else {
            println!("Not a selection");
            println!("Please Select B/s");
            continue;
        }
    }
    config
}

fn get_bulk_selections(vimmslair_helper: &VimmsLairHelper, mut config: Config) -> Config {
    println!("Press Enter to download all of Vimm's roms or select from the following of what systems you would like to download");
    println!("Enter 'd' when finished if choosing specific consoles\n");
    vimmslair_helper.print_console_list();
    loop {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        let trimmed = user_input.trim();
        if trimmed.is_empty() && config.selections.is_empty() {
            config.all = true;
            break;
        }
        if trimmed == "d" {
            break;
        }
        match trimmed.parse::<usize>() {
            Ok(num) => {
                let max = CONSOLES.len().saturating_sub(1);
                if num <= max {
                    config.selections.push(num);
                } else {
                    println!("Not a selection");
                    println!("Please select a value from the list");
                }
            }
            Err(_) => {
                println!("Please select a value from the list");
                continue;
            }
        }
    }
    config
}

fn get_extraction_status(mut config: Config) -> Config {
    println!("Would you like to automatically extract and delete archives after download? (Y/n)");
    println!("Default is 'y'");
    loop {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        let trimmed = user_input.trim();
        if trimmed.is_empty() {
            config.extract = true;
            break;
        }
        if trimmed.to_lowercase() == "y" {
            config.extract = true;
            break;
        }
        if trimmed.to_lowercase() == "n" {
            config.extract = false;
            break;
        }
        if trimmed.to_lowercase() != "n" && trimmed.to_lowercase() != "y" {
            println!("Not a selection");
            println!("Please Select Y/n");
            continue;
        }
    }
    config
}

fn print_general_search(roms: &[Rom]) {
    let mut max_selection = 18;
    let mut max_system = 6;
    let mut max_rom = 3;
    let mut max_region = 6;
    let mut max_version = 7;
    for rom in roms {
        max_rom = max_rom.max(rom.name.len());
        max_system = max_system.max(rom.system.len());
        max_region = max_region.max(rom.region.len());
        max_version = max_version.max(rom.version.len());
    }

    println!("\nSelect which roms you would like to download and then enter 'd'\n");
    println!(
        "{:<width$} {:<width2$} {:<width3$} {:<width4$} {}",
        "Selection Number",
        "System",
        "ROM",
        "Region",
        "Version",
        width = max_selection,
        width2 = max_system,
        width3 = max_rom,
        width4 = max_region
    );
    for (count, rom) in roms.iter().enumerate() {
        println!(
            "{:<width$} {:<width2$} {:<width3$} {:<width4$} {}",
            count,
            rom.system,
            rom.name,
            rom.region,
            rom.version,
            width = max_selection,
            width2 = max_system,
            width3 = max_rom,
            width4 = max_region
        );
    }
}

fn print_system_search(roms: &[Rom]) {
    let mut max_selection = 18;
    let mut max_system = 6;
    let mut max_rom = 3;
    let mut max_region = 6;
    let mut max_version = 7;
    for rom in roms {
        max_rom = max_rom.max(rom.name.len());
        max_system = max_system.max(rom.system.len());
        max_region = max_region.max(rom.region.len());
        max_version = max_version.max(rom.version.len());
    }

    println!("\nFound {} results:", roms.len());
    println!(
        "{:<width$} {:<width2$} {:<width3$} {:<width4$} {}",
        "Selection Number",
        "System",
        "ROM",
        "Region",
        "Version",
        width = max_selection,
        width2 = max_system,
        width3 = max_rom,
        width4 = max_region
    );
    for (count, rom) in roms.iter().enumerate() {
        println!(
            "{:<width$} {:<width2$} {:<width3$} {:<width4$} {}",
            count,
            rom.system,
            rom.name,
            rom.region,
            rom.version,
            width = max_selection,
            width2 = max_system,
            width3 = max_rom,
            width4 = max_region
        );
    }
}

fn print_search_results(roms: &[Rom]) {
    if !roms.is_empty() && roms[0].system.is_empty() {
        print_system_search(roms);
    } else {
        print_general_search(roms);
    }
}

fn get_search_result_input(roms: &[Rom]) -> Vec<usize> {
    let mut download_sel_roms: Vec<usize> = Vec::new();
    println!("\nSelect which roms you would like to download and then enter 'd'");
    loop {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        let trimmed = user_input.trim();
        if trimmed.is_empty() {
            println!("Please select a rom or press 'q' to quit program");
            continue;
        }
        if trimmed == "q" {
            std::process::exit(0);
        }
        if trimmed == "d" {
            break;
        }
        match trimmed.parse::<usize>() {
            Ok(num) => {
                if num < roms.len() {
                    download_sel_roms.push(num);
                } else {
                    println!("Not a selection");
                    println!("Please select a value from the list");
                }
            }
            Err(_) => {
                println!("Please select a value from the list");
                continue;
            }
        }
    }
    download_sel_roms
}

fn download_search_results(
    vimmslair_helper: &VimmsLairHelper,
    downloads: &[usize],
    roms: &[Rom],
    config: &Config,
    home_dir: &str,
) {
    for &x in downloads {
        let download_name = download_file(vimmslair_helper, &roms[x].download_url, home_dir);
        if config.extract && !download_name.is_empty() {
            extract_and_delete_search_results(home_dir, &download_name);
        }
    }
}

fn extract_file(path: &str, name: &str) {
    let full_path = Path::new(path).join(name);
    let re = Regex::new(r"(.+?)(\.[^.]*$|$)").unwrap();
    let base_filename = if let Some(caps) = re.captures(name) {
        caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default()
    } else {
        String::new()
    };
    let ext_re = Regex::new(r"(zip|7z)").unwrap();
    let file_type = if let Some(caps) = ext_re.captures(&full_path.to_string_lossy()) {
        caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default()
    } else {
        String::new()
    };
    let dir_path = create_directory_for_rom(&base_filename, path);
    if file_type.to_lowercase() == "zip" {
        if let Ok(file) = File::open(&full_path) {
            if let Ok(mut archive) = ZipArchive::new(file) {
                for i in 0..archive.len() {
                    if let Ok(mut file_in_archive) = archive.by_index(i) {
                        let outpath = Path::new(&dir_path).join(file_in_archive.mangled_name());
                        if file_in_archive.is_dir() {
                            let _ = fs::create_dir_all(&outpath);
                        } else {
                            if let Some(parent) = outpath.parent() {
                                let _ = fs::create_dir_all(parent);
                            }
                            if let Ok(mut outfile) = File::create(&outpath) {
                                use std::io::copy;
                                let _ = copy(&mut file_in_archive, &mut outfile);
                            }
                        }
                    }
                }
            }
        }
    }
    if file_type.to_lowercase() == "7z" {
        if let Ok(mut sz) = sevenz_rust::SevenZReader::open(&full_path, sevenz_rust::Password::from("")) {
                let _ = sz.for_each_entries(|entry, reader| {
                    let outpath = Path::new(&dir_path).join(entry.name());
                    if entry.is_directory() {
                        let _ = fs::create_dir_all(&outpath);
                    } else {
                        if let Some(parent) = outpath.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        if let Ok(mut outfile) = File::create(&outpath) {
                            use std::io::copy;
                            let _ = copy(reader, &mut outfile);
                        }
                    }
                    Ok(true)
                });
        }
    }
}

fn delete_file(path: &str, name: &str) {
    let full_path = Path::new(path).join(name);
    if let Err(e) = fs::remove_file(&full_path) {
        println!("Failed to delete {}: {}", full_path.display(), e);
    }
}

fn check_if_need_to_re_search() -> bool {
    let mut search = false;
    println!("Do you want to search again?(y/N)");
    loop {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        let trimmed = user_input.trim();
        if trimmed.is_empty() {
            break;
        }
        if trimmed.to_lowercase() == "y" {
            search = true;
            break;
        }
        if trimmed.to_lowercase() == "n" {
            break;
        }
        if trimmed.to_lowercase() != "n" && trimmed.to_lowercase() != "y" {
            println!("Not a selection");
            println!("Please Select y/N");
            continue;
        }
    }
    search
}

fn run_search(vimmslair_helper: &VimmsLairHelper, config: &Config) -> Vec<Rom> {
    get_search_section(vimmslair_helper, &config.query.search_selections)
}

fn create_directory_for_rom(name: &str, path: &str) -> String {
    let new_path = Path::new(path).join(name);
    if let Err(e) = fs::create_dir_all(&new_path) {
        println!("Directory already exists: {}", new_path.display());
        println!("{}", e);
    }
    new_path.to_string_lossy().to_string()
}

fn run_search_loop(vimmslair_helper: &VimmsLairHelper, mut config: Config, home_dir: &str) {
    loop {
        config = get_search_selection(vimmslair_helper, config);
        let roms = run_search(vimmslair_helper, &config);
        print_search_results(&roms);
        let restart = check_if_need_to_re_search();
        if restart {
            continue;
        }
        let downloads = get_search_result_input(&roms);
        download_search_results(vimmslair_helper, &downloads, &roms, &config, home_dir);
        println!("Done!");
        let restart = check_if_need_to_re_search();
        if restart {
            continue;
        } else {
            std::process::exit(0);
        }
    }
}

fn extract_and_delete_search_results(path: &str, download: &str) {
    extract_file(path, download);
    println!("Finished extracting {}!", download);
    delete_file(path, download);
}

fn download_bulk_roms(vimmslair_helper: &VimmsLairHelper, config: &Config, roms: &mut Vec<BulkSystemRoms>) {
    for system in roms.iter_mut() {
        println!("Starting to download all roms for the {}!", system.system_name);
        for section in system.sections.iter_mut() {
            for rom in section.roms.iter() {
                let download_name = download_file(vimmslair_helper, &rom.download_url, &section.path);
                if download_name.is_empty() {
                    println!("Failed to download {}", rom.name);
                    continue;
                }
                if config.extract {
                    extract_and_delete_search_results(&section.path, &download_name);
                }
            }
        }
    }
}

fn run_selected_program(vimmslair_helper: &VimmsLairHelper, mut config: Config, home_dir: &str) {
    if config.bulk_mode {
        config = get_bulk_selections(vimmslair_helper, config);
        let mut roms = if config.all {
            get_every_system_roms(vimmslair_helper)
        } else {
            get_selected_system_bulk_roms(vimmslair_helper, &config)
        };
        vimmslair_helper.generate_path_to_bulk_roms(&mut roms, home_dir);
        vimmslair_helper.create_directory_structure(&config, home_dir);
        download_bulk_roms(vimmslair_helper, &config, &mut roms);
        std::process::exit(0);
    }
    if config.search_mode {
        run_search_loop(vimmslair_helper, config, home_dir);
    }
}

fn main() {
    let vimmslair_helper = VimmsLairHelper::new();
    let exe_path = std::env::current_exe().unwrap();
    let home_dir = exe_path.parent().unwrap().parent().unwrap().to_string_lossy().to_string();
    vimmslair_helper.print_welcome();
    let config = get_program_mode();
    let config = get_extraction_status(config);
    run_selected_program(&vimmslair_helper, config, &home_dir);
}
