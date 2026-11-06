use horizont_downloader::{search_content, ContentType};

#[test]
fn test_search_resourcepacks() {
    let results = search_content("faithful", ContentType::ResourcePack, 3);
    assert!(results.is_ok(), "Search should succeed: {:?}", results.err());
    let list = results.unwrap();
    println!("Found {} resource packs:", list.len());
    for p in &list {
        println!(" - {} by {} ({} downloads)", p.title, p.author, p.downloads);
    }
    assert!(!list.is_empty(), "Should find at least 1 result for 'faithful'");
}

#[test]
fn test_search_shaders() {
    let results = search_content("bsl", ContentType::ShaderPack, 3);
    assert!(results.is_ok(), "Search should succeed: {:?}", results.err());
    let list = results.unwrap();
    println!("Found {} shaders:", list.len());
    for p in &list {
        println!(" - {} by {} ({} downloads)", p.title, p.author, p.downloads);
    }
    assert!(!list.is_empty(), "Should find at least 1 result for 'bsl'");
}
