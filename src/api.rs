use crate::models::{Console, Rom};
use base64::{Engine as _, engine::general_purpose};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Mutex;

const BASE: &str = "https://vimm.net";
const DL_BASE: &str = "https://dl2.vimm.net";

lazy_static! {
    static ref IMAGE_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref GAME_DETAIL_CACHE: Mutex<HashMap<String, Rom>> = Mutex::new(HashMap::new());
    static ref SECTION_CACHE: Mutex<HashMap<String, Vec<Rom>>> = Mutex::new(HashMap::new());
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}

pub fn consoles() -> Vec<Console> {
    vec![
        Console { name: "Atari 7800".into(), slug: "Atari7800".into() },
        Console { name: "Nintendo NES".into(), slug: "NES".into() },
        Console { name: "Super Nintendo".into(), slug: "SNES".into() },
        Console { name: "Nintendo 64".into(), slug: "N64".into() },
        Console { name: "Wii".into(), slug: "Wii".into() },
        Console { name: "Wii Ware".into(), slug: "WiiWare".into() },
        Console { name: "Gamecube".into(), slug: "GameCube".into() },
        Console { name: "Master System".into(), slug: "SMS".into() },
        Console { name: "Mega Drive".into(), slug: "Genesis".into() },
        Console { name: "Saturn".into(), slug: "Saturn".into() },
        Console { name: "Dreamcast".into(), slug: "Dreamcast".into() },
        Console { name: "Playstation".into(), slug: "PS1".into() },
        Console { name: "Playstation 2".into(), slug: "PS2".into() },
        Console { name: "Playstation 3".into(), slug: "PS3".into() },
        Console { name: "Xbox".into(), slug: "Xbox".into() },
        Console { name: "Game Boy".into(), slug: "GB".into() },
        Console { name: "Game Boy Color".into(), slug: "GBC".into() },
        Console { name: "Game Boy Advanced".into(), slug: "GBA".into() },
        Console { name: "Nintendo DS".into(), slug: "DS".into() },
        Console { name: "Playstation Portable".into(), slug: "PSP".into() },
    ]
}

fn build_search_url(query: &str, system: Option<&str>) -> String {
    let mut url = if let Some(sys) = system {
        format!("{}/vault/?p=list&action=filters&system={}&q={}", BASE, sys, query)
    } else {
        format!("{}/vault/?p=list&action=filters&q={}", BASE, query)
    };
    url.push_str("&version=new&countries%5B%5D=8&countries%5B%5D=14&countries%5B%5D=26");
    url
}

fn extract_text(el: scraper::ElementRef) -> String {
    let mut s = String::new();
    for t in el.text() {
        s.push_str(t);
    }
    s.trim().to_string()
}

fn parse_table_rows(document: &Html) -> Vec<Rom> {
    let table_sel = Selector::parse("table.rounded.centered.cellpadding1.hovertable.striped").unwrap();
    let row_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let img_sel = Selector::parse("img").unwrap();
    let b_sel = Selector::parse("b.redBorder").unwrap();

    let mut roms = Vec::new();
    let Some(table) = document.select(&table_sel).next() else { return roms };

    for row in table.select(&row_sel) {
        if row.select(&Selector::parse("th").unwrap()).next().is_some() {
            continue;
        }
        let tds: Vec<_> = row.select(&td_sel).collect();
        let Some(name_td) = tds.first().copied() else { continue };
        let region_td = tds.get(1).copied();
        let version_td = tds.get(2).copied();

        let Some(rom_a) = name_td.select(&a_sel).find(|a| {
            !extract_text(*a).is_empty()
        }) else { continue };

        let mut name = extract_text(rom_a);
        if let Some(rb) = name_td.select(&b_sel).next() {
            if let Some(title) = rb.value().attr("title") {
                name.push(' ');
                name.push_str(title);
            }
        }

        let href = rom_a.value().attr("href").unwrap_or("").to_string();
        let page_url = format!("{}{}", BASE, href);
        let id = href.split('/').last().unwrap_or("").to_string();
        let download_url = format!("{}?mediaId={}", DL_BASE, id);
        let image_url = format!("https://dl.vimm.net/image.php?type=box&id={}", id);

        let region = if let Some(rtd) = region_td {
            let imgs: Vec<_> = rtd.select(&img_sel).collect();
            if !imgs.is_empty() {
                let mut parts = Vec::new();
                for (i, img) in imgs.iter().enumerate() {
                    parts.push(img.value().attr("title").unwrap_or("").to_string());
                    if i != imgs.len() - 1 { parts.push(" / ".into()); }
                }
                parts.concat()
            } else {
                extract_text(rtd)
            }
        } else {
            String::new()
        };

        let version = if let Some(vtd) = version_td {
            extract_text(vtd)
        } else {
            String::new()
        };

        let mut region_flags = Vec::new();
        if let Some(rtd) = region_td {
            for img in rtd.select(&img_sel) {
                if let Some(title) = img.value().attr("title") {
                    region_flags.push(title.to_string());
                }
            }
        }

        roms.push(Rom {
            id,
            name,
            page_url,
            download_url,
            system: String::new(),
            region,
            version,
            region_flags,
            year: String::new(),
            players: String::new(),
            size: String::new(),
            rating: String::new(),
            description: String::new(),
            image_url,
            screen_url: String::new(),
            crc: String::new(),
            md5: String::new(),
            sha1: String::new(),
            graphics: String::new(),
            sound: String::new(),
            gameplay: String::new(),
            overall: String::new(),
            publisher: String::new(),
            serial: String::new(),
            file_name: String::new(),
            verified: String::new(),
        });
    }
    roms
}

pub async fn search(query: &str, system: Option<&str>) -> Result<Vec<Rom>, String> {
    let url = build_search_url(query, system);
    let resp = client().get(&url).send().await.map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    let doc = Html::parse_document(&text);
    Ok(parse_table_rows(&doc))
}

pub async fn fetch_section(console: &str, section: &str) -> Result<Vec<Rom>, String> {
    let cache_key = format!("{}:{}", console, section);

    {
        let cache = SECTION_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(cached.clone());
        }
    }

    let url = format!("{}/vault/?p=list&action=filters&system={}&section={}&version=new", BASE, console, section);
    let resp = client().get(&url).send().await.map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    let doc = Html::parse_document(&text);
    let roms = parse_table_rows(&doc);

    {
        let mut cache = SECTION_CACHE.lock().unwrap();
        cache.insert(cache_key, roms.clone());
    }

    Ok(roms)
}

pub async fn fetch_game_detail(id: &str) -> Result<Rom, String> {
    {
        let cache = GAME_DETAIL_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(id) {
            return Ok(cached.clone());
        }
    }

    let url = format!("{}/vault/{}", BASE, id);
    let resp = client().get(&url).send().await.map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    let doc = Html::parse_document(&text);

    let desc_sel = Selector::parse("meta[name=\"description\"]").unwrap();
    let description = doc.select(&desc_sel)
        .next()
        .and_then(|m| m.value().attr("content"))
        .unwrap_or("")
        .to_string();

    let form_sel = Selector::parse("#dl_form").unwrap();
    let input_sel = Selector::parse("input[name=\"mediaId\"]").unwrap();
    let download_url = if let Some(form) = doc.select(&form_sel).next() {
        if let Some(input) = form.select(&input_sel).next() {
            let media_id = input.value().attr("value").unwrap_or(id);
            format!("{}?mediaId={}", DL_BASE, media_id)
        } else {
            format!("{}?mediaId={}", DL_BASE, id)
        }
    } else {
        format!("{}?mediaId={}", DL_BASE, id)
    };

    let mut rom = Rom::new_basic(String::new(), url.clone(), download_url);
    rom.id = id.to_string();
    rom.description = description;
    rom.image_url = format!("https://dl.vimm.net/image.php?type=box&id={}", id);
    rom.screen_url = format!("https://dl.vimm.net/image.php?type=screen&id={}", id);

    let title_sel = Selector::parse("h2").unwrap();
    if let Some(h2) = doc.select(&title_sel).next() {
        rom.name = extract_text(h2);
    }

    // Parse the detail table rows
    let tr_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let img_sel = Selector::parse("img").unwrap();
    let span_sel = Selector::parse("span").unwrap();

    for tr in doc.select(&tr_sel) {
        let tds: Vec<_> = tr.select(&td_sel).collect();
        if tds.len() < 3 {
            continue;
        }
        let label = extract_text(tds[0]);
        let value_td = tds[2];
        let value = extract_text(value_td);

        match label.as_str() {
            "Region" => {
                let mut parts = Vec::new();
                for img in value_td.select(&img_sel) {
                    if let Some(title) = img.value().attr("title") {
                        parts.push(title.to_string());
                    }
                }
                rom.region = if parts.is_empty() { value } else { parts.join(" / ") };
            }
            "Players" => rom.players = value,
            "Year" => rom.year = value,
            "Cart size" | "Disc size" | "File size" => rom.size = value,
            "Graphics" => rom.graphics = value,
            "Sound" => rom.sound = value,
            "Gameplay" => rom.gameplay = value,
            "Overall" => rom.overall = value.split_whitespace().next().unwrap_or(&value).to_string(),
            "CRC" => {
                for span in value_td.select(&span_sel) {
                    if let Some(id_attr) = span.value().attr("id") {
                        if id_attr == "data-crc" {
                            rom.crc = extract_text(span);
                        }
                    }
                }
            }
            "MD5" => {
                for span in value_td.select(&span_sel) {
                    if let Some(id_attr) = span.value().attr("id") {
                        if id_attr == "data-md5" {
                            rom.md5 = extract_text(span);
                        }
                    }
                }
            }
            "SHA1" => {
                for span in value_td.select(&span_sel) {
                    if let Some(id_attr) = span.value().attr("id") {
                        if id_attr == "data-sha1" {
                            rom.sha1 = extract_text(span);
                        }
                    }
                }
            }
            "Verified" => {
                for span in value_td.select(&span_sel) {
                    if let Some(id_attr) = span.value().attr("id") {
                        if id_attr == "data-date" {
                            rom.verified = extract_text(span);
                        }
                    }
                }
            }
            "Version" => rom.version = value,
            "Publisher" => rom.publisher = value,
            _ if label.starts_with("Serial #") => rom.serial = value,
            _ => {}
        }
    }

    // Extract file name from data-good-title canvas
    let good_title_sel = Selector::parse("#data-good-title canvas").unwrap();
    if let Some(canvas) = doc.select(&good_title_sel).next() {
        if let Some(data_v) = canvas.value().attr("data-v") {
            if let Ok(decoded) = general_purpose::STANDARD.decode(data_v.trim()) {
                if let Ok(s) = String::from_utf8(decoded) {
                    rom.file_name = s;
                }
            }
        }
    }

    {
        let mut cache = GAME_DETAIL_CACHE.lock().unwrap();
        cache.insert(id.to_string(), rom.clone());
    }

    Ok(rom)
}

pub async fn fetch_image_data_url(id: &str, image_type: &str) -> Result<String, String> {
    let cache_key = format!("{}:{}", image_type, id);

    {
        let cache = IMAGE_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(cached.clone());
        }
    }

    let url = format!("https://dl.vimm.net/image.php?type={}&id={}", image_type, id);
    let resp = client()
        .get(&url)
        .header("Referer", format!("https://vimm.net/vault/{}", id))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let content_type = resp.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/png")
        .to_string();

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let b64 = general_purpose::STANDARD.encode(&bytes);
    let result = format!("data:{};base64,{}", content_type, b64);

    {
        let mut cache = IMAGE_CACHE.lock().unwrap();
        cache.insert(cache_key, result.clone());
    }

    Ok(result)
}

pub async fn preload_game(id: &str) {
    if fetch_game_detail(id).await.is_ok() {
        let _ = tokio::join!(
            fetch_image_data_url(id, "box"),
            fetch_image_data_url(id, "screen"),
        );
    }
}

pub async fn do_download(url: &str, path: &str) -> Result<(), String> {
    let resp = client().get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    tokio::fs::write(path, &bytes).await.map_err(|e| e.to_string())?;
    Ok(())
}
