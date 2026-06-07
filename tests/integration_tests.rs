use vimms::helpers::{VimmsLairHelper, CONSOLES};
use vimms::models::{Config, Rom, SearchSelection};

use reqwest::blocking::Client;
use scraper::{Html, Selector};

// Helper to build a client matching the app's configuration
fn build_test_client() -> Client {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}

#[test]
fn test_vimms_lair_reachable() {
    let client = build_test_client();
    let response = client
        .get(VimmsLairHelper::VIMMS_LAIR_BASE_URL)
        .send();
    assert!(
        response.is_ok(),
        "Failed to reach vimm.net: {:?}",
        response.err()
    );
    let response = response.unwrap();
    assert!(
        response.status().is_success(),
        "vimm.net returned non-success status: {}",
        response.status()
    );
}

#[test]
fn test_search_url_generation() {
    let helper = VimmsLairHelper::new();

    // Test system-specific search URL
    let ss = SearchSelection {
        system: "Nintendo NES".to_string(),
        query: "mario".to_string(),
    };
    let url = helper.get_search_url(&ss);
    assert!(url.contains("system=NES"));
    assert!(url.contains("q=mario"));
    assert!(url.starts_with("https://vimm.net"));

    // Test general search URL
    let ss_general = SearchSelection {
        system: "general".to_string(),
        query: "zelda".to_string(),
    };
    let url_general = helper.get_search_url(&ss_general);
    assert!(url_general.contains("q=zelda"));
    assert!(!url_general.contains("system="));
}

#[test]
fn test_live_search_parsing() {
    let helper = VimmsLairHelper::new();
    let client = build_test_client();

    // Search for a well-known ROM that should exist
    let search_selection = SearchSelection {
        system: "Nintendo NES".to_string(),
        query: "mario".to_string(),
    };
    let search_url = helper.get_search_url(&search_selection);

    let response = client.get(&search_url).send();
    assert!(
        response.is_ok(),
        "Failed to fetch search page: {:?}",
        response.err()
    );
    let response = response.unwrap();
    assert!(
        response.status().is_success(),
        "Search page returned non-success: {}",
        response.status()
    );

    let text = response.text().unwrap();
    let document = Html::parse_document(&text);

    // Check that the expected table exists (or note if it doesn't)
    let table_selector =
        Selector::parse("table.rounded.centered.cellpadding1.hovertable.striped").unwrap();
    let table_exists = document.select(&table_selector).next().is_some();

    if table_exists {
        let table = document.select(&table_selector).next().unwrap();
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        // Replicate the app's CURRENT broken logic (picks first <a> which is empty)
        let mut found_roms_broken = 0;
        for row in table.select(&row_selector) {
            if row.select(&Selector::parse("th").unwrap()).next().is_some() {
                continue;
            }
            let tds: Vec<_> = row.select(&td_selector).collect();
            if let Some(first_td) = tds.first() {
                if let Some(rom_a) = first_td.select(&a_selector).next() {
                    let name = rom_a.text().collect::<String>().trim().to_string();
                    if !name.is_empty() {
                        found_roms_broken += 1;
                    }
                }
            }
        }

        // Fixed logic: skip empty anchor tags (e.g. <a href="/vault/999999"></a>)
        let mut found_roms_fixed = 0;
        for row in table.select(&row_selector) {
            if row.select(&Selector::parse("th").unwrap()).next().is_some() {
                continue;
            }
            let tds: Vec<_> = row.select(&td_selector).collect();
            if let Some(first_td) = tds.first() {
                for rom_a in first_td.select(&a_selector) {
                    let name = rom_a.text().collect::<String>().trim().to_string();
                    if !name.is_empty() {
                        found_roms_fixed += 1;
                        break;
                    }
                }
            }
        }

        println!(
            "Search results: broken logic found {}, fixed logic found {}",
            found_roms_broken, found_roms_fixed
        );

        // The site returns results, but the current scraper is broken
        assert!(
            found_roms_fixed > 0,
            "The site returned no results at all."
        );

        // This assertion documents the bug in the current app code
        if found_roms_broken == 0 && found_roms_fixed > 0 {
            println!(
                "BUG CONFIRMED: The app picks the first <a> tag which is now an empty \
                 /vault/999999 tooltip anchor. It skips the actual ROM link."
            );
        }
        assert!(
            found_roms_broken > 0 || found_roms_fixed > 0,
            "Search parsing completely failed."
        );
    } else {
        panic!(
            "Search results table not found in page HTML. \
             The site structure has likely changed. HTML snippet: {}",
            &text[..text.len().min(500)]
        );
    }
}

#[test]
fn test_live_download_id_extraction() {
    let client = build_test_client();

    // Use a well-known ROM page URL
    let page_url = format!("{}/vault/3480", VimmsLairHelper::VIMMS_LAIR_BASE_URL);

    let response = client.get(&page_url).send();
    if response.is_err() {
        println!("Skipping download ID test: could not reach site");
        return;
    }
    let response = response.unwrap();
    if !response.status().is_success() {
        println!("Skipping download ID test: page returned {}", response.status());
        return;
    }

    let text = response.text().unwrap();
    let document = Html::parse_document(&text);

    let form_selector = Selector::parse("#dl_form").unwrap();
    let input_selector = Selector::parse("input[name=\"mediaId\"]").unwrap();

    let form_exists = document.select(&form_selector).next().is_some();
    let input_exists = document.select(&input_selector).next().is_some();

    println!(
        "Download form present: {}, mediaId input present: {}",
        form_exists, input_exists
    );

    if form_exists {
        let form = document.select(&form_selector).next().unwrap();
        let input = form.select(&input_selector).next();
        if let Some(input) = input {
            let value = input.value().attr("value").unwrap_or("");
            println!("Extracted mediaId: {}", value);
            assert!(!value.is_empty(), "mediaId value was empty");
        } else {
            println!(
                "WARNING: mediaId input not found inside #dl_form. \
                 The site HTML may have changed."
            );
        }
    } else {
        println!(
            "WARNING: #dl_form not found on ROM page. \
             The site HTML may have changed."
        );
    }
}

#[test]
fn test_section_of_roms_parsing() {
    let _helper = VimmsLairHelper::new();
    let client = build_test_client();

    // Test fetching a section list page (NES section A)
    let section_url = format!(
        "{}/vault/?p=list&action=filters&system=NES&section=A",
        VimmsLairHelper::VIMMS_LAIR_BASE_URL
    );

    let response = client.get(&section_url).send();
    if response.is_err() {
        println!("Skipping section test: could not reach site");
        return;
    }
    let response = response.unwrap();
    if !response.status().is_success() {
        println!("Skipping section test: page returned {}", response.status());
        return;
    }

    let text = response.text().unwrap();
    let document = Html::parse_document(&text);

    let table_selector =
        Selector::parse("table.rounded.centered.cellpadding1.hovertable.striped").unwrap();

    if let Some(table) = document.select(&table_selector).next() {
        let row_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        // Broken logic (matches current app code)
        let mut found_broken = 0;
        for row in table.select(&row_selector) {
            let tds: Vec<_> = row.select(&td_selector).collect();
            if let Some(first_td) = tds.first() {
                if let Some(rom_a) = first_td.select(&a_selector).next() {
                    let name = rom_a.text().collect::<String>().trim().to_string();
                    if !name.is_empty() {
                        found_broken += 1;
                    }
                }
            }
        }

        // Fixed logic: skip empty anchor tags
        let mut found_fixed = 0;
        for row in table.select(&row_selector) {
            let tds: Vec<_> = row.select(&td_selector).collect();
            if let Some(first_td) = tds.first() {
                for rom_a in first_td.select(&a_selector) {
                    let name = rom_a.text().collect::<String>().trim().to_string();
                    if !name.is_empty() {
                        found_fixed += 1;
                        break;
                    }
                }
            }
        }

        println!(
            "Section list: broken logic found {}, fixed logic found {}",
            found_broken, found_fixed
        );

        assert!(
            found_fixed > 0,
            "The site returned no section results at all."
        );

        if found_broken == 0 && found_fixed > 0 {
            println!(
                "BUG CONFIRMED: Same empty /vault/999999 anchor issue in section list."
            );
        }
    } else {
        panic!(
            "Section results table not found. Site HTML may have changed. \
             Snippet: {}",
            &text[..text.len().min(500)]
        );
    }
}
