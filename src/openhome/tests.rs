use super::*;

fn log(_s: String) {}

#[test]
fn renderer() {
    let renderer = Renderer::new();
    let (host, port) = renderer.parse_url("http://192.168.1.26/", &log);
    assert_eq!(host, "192.168.1.26");
    assert_eq!(port, 80); // default port
    let (host, port) = renderer.parse_url("http://192.168.1.26:12345/", &log);
    assert_eq!(host, "192.168.1.26");
    assert_eq!(port, 12345); // other port
}
#[test]
fn control_url_harman_kardon() {
    let mut url = "Avcontrol.url".to_string();
    if !url.is_empty() && !url.starts_with('/') {
        url.insert(0, '/');
    }
    assert_eq!(url, "/Avcontrol.url");
    url = "/Avcontrol.url".to_string();
    if !url.is_empty() && !url.starts_with('/') {
        url.insert(0, '/');
    }
    assert_eq!(url, "/Avcontrol.url");
    url = "".to_string();
    if !url.is_empty() && !url.starts_with('/') {
        url.insert(0, '/');
    }
    assert_eq!(url, "");
    url = "A/.url".to_string();
    if !url.is_empty() && !url.starts_with('/') {
        url.insert(0, '/');
    }
    assert_eq!(url, "/A/.url");
}
