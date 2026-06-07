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

pub fn client() -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36"
            .parse()
            .unwrap(),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        "en-US,en;q=0.5".parse().unwrap(),
    );

    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .default_headers(headers)
        .build()
        .unwrap()
}

pub fn consoles() -> Vec<Console> {
    vec![
        Console { name: "Atari 2600".into(), slug: "Atari2600".into() },
        Console { name: "Atari 5200".into(), slug: "Atari5200".into() },
        Console { name: "Atari 7800".into(), slug: "Atari7800".into() },
        Console { name: "Atari Jaguar".into(), slug: "Jaguar".into() },
        Console { name: "Jaguar CD".into(), slug: "JaguarCD".into() },
        Console { name: "Atari Lynx".into(), slug: "Lynx".into() },
        Console { name: "CD-i".into(), slug: "CDi".into() },
        Console { name: "Nintendo".into(), slug: "NES".into() },
        Console { name: "Super Nintendo".into(), slug: "SNES".into() },
        Console { name: "Nintendo 64".into(), slug: "N64".into() },
        Console { name: "GameCube".into(), slug: "GameCube".into() },
        Console { name: "Wii".into(), slug: "Wii".into() },
        Console { name: "WiiWare".into(), slug: "WiiWare".into() },
        Console { name: "Game Boy".into(), slug: "GB".into() },
        Console { name: "Game Boy Color".into(), slug: "GBC".into() },
        Console { name: "Game Boy Advance".into(), slug: "GBA".into() },
        Console { name: "Nintendo DS".into(), slug: "DS".into() },
        Console { name: "Nintendo 3DS".into(), slug: "3DS".into() },
        Console { name: "Virtual Boy".into(), slug: "VB".into() },
        Console { name: "Master System".into(), slug: "SMS".into() },
        Console { name: "Genesis".into(), slug: "Genesis".into() },
        Console { name: "Sega 32X".into(), slug: "32X".into() },
        Console { name: "Sega CD".into(), slug: "SegaCD".into() },
        Console { name: "Saturn".into(), slug: "Saturn".into() },
        Console { name: "Dreamcast".into(), slug: "Dreamcast".into() },
        Console { name: "Game Gear".into(), slug: "GG".into() },
        Console { name: "TurboGrafx-16".into(), slug: "TG16".into() },
        Console { name: "TurboGrafx-CD".into(), slug: "TGCD".into() },
        Console { name: "PlayStation".into(), slug: "PS1".into() },
        Console { name: "PlayStation 2".into(), slug: "PS2".into() },
        Console { name: "PlayStation 3".into(), slug: "PS3".into() },
        Console { name: "PS Portable".into(), slug: "PSP".into() },
        Console { name: "Xbox".into(), slug: "Xbox".into() },
        Console { name: "Xbox 360".into(), slug: "Xbox360".into() },
        Console { name: "Xbox 360 (Digital)".into(), slug: "X360-D".into() },
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

fn parse_table_rows(document: &Html, system: &str) -> Vec<Rom> {
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
            system: system.to_string(),
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
    Ok(parse_table_rows(&doc, system.unwrap_or("")))
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
    let roms = parse_table_rows(&doc, console);

    {
        let mut cache = SECTION_CACHE.lock().unwrap();
        cache.insert(cache_key, roms.clone());
    }

    Ok(roms)
}

pub async fn fetch_all_sections(console: &str) -> Result<Vec<Rom>, String> {
    let sections: Vec<String> = std::iter::once("number".to_string())
        .chain(('A'..='Z').map(|c| c.to_string()))
        .collect();

    let mut join_set = tokio::task::JoinSet::new();
    for sec in sections {
        let slug = console.to_string();
        join_set.spawn(async move { fetch_section(&slug, &sec).await });
    }

    let mut all_roms = Vec::new();
    while let Some(res) = join_set.join_next().await {
        if let Ok(Ok(roms)) = res {
            all_roms.extend(roms);
        }
    }

    all_roms.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(all_roms)
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

    let title_sel = Selector::parse("title").unwrap();
    let system_from_title = doc.select(&title_sel)
        .next()
        .map(|t| extract_text(t))
        .and_then(|title| {
            title.rfind('(').and_then(|start| {
                title.rfind(')').and_then(|end| {
                    if end > start {
                        Some(title[start + 1..end].to_string())
                    } else {
                        None
                    }
                })
            })
        })
        .unwrap_or_default();

    let form_sel = Selector::parse("#dl_form").unwrap();
    let input_sel = Selector::parse("input[name=\"mediaId\"]").unwrap();
    let download_url = if let Some(form) = doc.select(&form_sel).next() {
        let action = form.value().attr("action").unwrap_or("");
        let action_url = if action.is_empty() {
            DL_BASE.to_string()
        } else if action.starts_with("http") {
            action.to_string()
        } else if action.starts_with("//") {
            format!("https:{}", action)
        } else if action.starts_with('/') {
            format!("{}{}", BASE, action)
        } else {
            format!("{}/{}", DL_BASE, action)
        };
        if let Some(input) = form.select(&input_sel).next() {
            let media_id = input.value().attr("value").unwrap_or(id);
            format!("{}?mediaId={}", action_url, media_id)
        } else {
            format!("{}?mediaId={}", action_url, id)
        }
    } else {
        format!("{}?mediaId={}", DL_BASE, id)
    };

    let mut rom = Rom::new_basic(String::new(), url.clone(), download_url);
    rom.id = id.to_string();
    rom.system = system_from_title;
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

pub async fn download_image(id: &str, image_type: &str, path: &str) -> Result<(), String> {
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

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    tokio::fs::write(path, &bytes).await.map_err(|e| e.to_string())?;
    Ok(())
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
    if let Some(parent) = std::path::Path::new(path).parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    tokio::fs::write(path, &bytes).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn extract_zip(zip_path: &str, out_dir: &str) -> Result<(), String> {
    let zip_path = zip_path.to_string();
    let out_dir = out_dir.to_string();
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = std::path::Path::new(&out_dir).join(file.mangled_name());
            if file.is_dir() {
                std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
                let mut outfile = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(())
}

fn extract_media_id(url: &str) -> Option<String> {
    url.split("mediaId=").nth(1).map(|s| s.split('&').next().unwrap_or(s).to_string())
}

pub async fn do_download_with_progress(
    client: &reqwest::Client,
    url: &str,
    path: &str,
    page_url: &str,
    mut on_progress: impl FnMut(f32),
) -> Result<(), String> {
    tracing::info!("[download] visiting page: {}", page_url);
    let page_resp = client.get(page_url).send().await.map_err(|e| e.to_string())?;
    tracing::info!("[download] page response: {}", page_resp.status());

    tracing::info!("[download] requesting file: {}", url);
    let resp = client
        .get(url)
        .header("referer", page_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    tracing::info!("[download] download response: {}", resp.status());

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::error!("[download] failed: status={}, body={}", status, body);
        return Err(format!("HTTP {} - {}", status, body));
    }

    let total = resp.content_length().unwrap_or(0) as f64;
    let mut stream = resp.bytes_stream();

    if let Some(parent) = std::path::Path::new(path).parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }

    let mut file = tokio::fs::File::create(path).await.map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        if total > 0.0 {
            on_progress((downloaded as f64 / total) as f32);
        }
    }

    on_progress(1.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "hits external network"]
    async fn test_download_with_progress_writes_file() {
        let client = client();
        let url = "https://httpbin.org/bytes/100?mediaId=test123";
        let path = std::env::temp_dir().join("vimms_test_download.bin");
        let path_str = path.to_string_lossy().to_string();

        let mut progress_values = Vec::new();
        let result = do_download_with_progress(
            &client,
            url,
            &path_str,
            "https://httpbin.org/",
            |p| progress_values.push(p),
        )
        .await;

        assert!(result.is_ok(), "download failed: {:?}", result.err());
        assert!(path.exists(), "file was not created");

        let meta = tokio::fs::metadata(&path).await.unwrap();
        assert_eq!(meta.len(), 100, "downloaded file size mismatch");

        assert!(!progress_values.is_empty(), "progress callback was never called");
        assert!(progress_values.last().unwrap() >= &0.99, "progress did not reach near 1.0");

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn test_download_with_progress_bad_url_fails() {
        let client = client();
        let path = std::env::temp_dir().join("vimms_test_download_fail.bin");
        let path_str = path.to_string_lossy().to_string();

        let result = do_download_with_progress(
            &client,
            "https://invalid.invalid.invalid/file?mediaId=123",
            &path_str,
            "https://invalid.invalid.invalid/",
            |_p| {},
        )
        .await;

        assert!(result.is_err(), "expected download to fail with bad url");
        assert!(!path.exists(), "file should not be created on failure");
    }

    #[test]
    fn test_extract_media_id() {
        assert_eq!(
            extract_media_id("https://dl2.vimm.net?mediaId=87558"),
            Some("87558".to_string())
        );
        assert_eq!(
            extract_media_id("https://dl.vimm.net/download?mediaId=abc&other=x"),
            Some("abc".to_string())
        );
        assert_eq!(extract_media_id("https://dl2.vimm.net"), None);
    }
}
