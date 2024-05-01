use std::collections::HashMap;

pub fn extract_query_string(url: &str) -> Option<HashMap<String, String>> {
    let url_parts = url.split('?').collect::<Vec<&str>>();
    if url_parts.len() == 2 {
        let query_string = url_parts[1];
        let query_pairs = query_string.split('&').collect::<Vec<&str>>();
        let mut query_map = HashMap::new();
        for query_pair in query_pairs {
            let pair_parts = query_pair.split('=').collect::<Vec<&str>>();
            if pair_parts.len() == 2 {
                query_map.insert(pair_parts[0].to_ascii_lowercase().to_string(), pair_parts[1].to_ascii_lowercase().to_string());
            }
        }
        Some(query_map)
    } else {
        None
    }
}

pub fn clear_query_string(url: &mut String) {
    let url_parts = url.split('?').collect::<Vec<&str>>();
    if url_parts.len() >= 2 {
        *url = url_parts[0].to_string();
    }
}