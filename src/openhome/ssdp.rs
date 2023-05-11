use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use log::{debug, error, info};
use xml::{reader::XmlEvent, EventReader};

use crate::{openhome::templates::SSDP_DISCOVER_MSG, CONFIG};

use super::{AvService, Renderer, SupportedProtocols};

//
// SSDP UPNP service discovery
//
// returns a list of all AVTransport DLNA and Openhome rendering devices
//
pub fn discover(
    rmap: &HashMap<String, Renderer>,
    logger: &dyn Fn(String),
) -> Option<Vec<Renderer>> {
    debug!("SSDP discovery started");

    const OH_DEVICE: &str = "urn:av-openhome-org:service:Product:1";
    const AV_DEVICE: &str = "urn:schemas-upnp-org:service:RenderingControl:1";
    const DEFAULT_SEARCH_TTL: u32 = 2;

    // get the address of the selected interface
    let local_addr = CONFIG.read().last_network.parse().unwrap();
    let bind_addr = SocketAddr::new(local_addr, 0);
    let socket = UdpSocket::bind(bind_addr).unwrap();
    socket.set_broadcast(true).unwrap();
    socket.set_multicast_ttl_v4(DEFAULT_SEARCH_TTL).unwrap();

    // broadcast the M-SEARCH message (MX is 3 secs) and collect responses
    let mut oh_devices: Vec<(String, SocketAddr)> = Vec::new();
    let mut av_devices: Vec<(String, SocketAddr)> = Vec::new();
    let mut devices: Vec<(String, SocketAddr)> = Vec::new();
    //  SSDP UDP broadcast address
    let broadcast_address: SocketAddr = ([239, 255, 255, 250], 1900).into();
    let msg = SSDP_DISCOVER_MSG.replace("{device_type}", OH_DEVICE);
    socket.send_to(msg.as_bytes(), broadcast_address).unwrap();
    let msg = SSDP_DISCOVER_MSG.replace("{device_type}", AV_DEVICE);
    socket.send_to(msg.as_bytes(), broadcast_address).unwrap();
    // collect the responses and remeber all new renderers
    let start = Instant::now();
    loop {
        let duration = start.elapsed().as_millis() as u64;
        // keep capturing responses for 3.1 seconds
        if duration >= 3100 {
            break;
        }
        let max_wait_time = 3100 - duration;
        socket
            .set_read_timeout(Some(Duration::from_millis(max_wait_time)))
            .unwrap();
        let mut buf: [u8; 2048] = [0; 2048];
        let resp: String;
        match socket.recv_from(&mut buf) {
            Ok((received, from)) => {
                resp = std::str::from_utf8(&buf[0..received]).unwrap().to_string();
                debug!(
                    "UDP response at {} from {}: \r\n{}",
                    start.elapsed().as_millis(),
                    from,
                    resp
                );
                let response: Vec<&str> = resp.split("\r\n").collect();
                if !response.is_empty() {
                    let status_code = response[0]
                        .trim_start_matches("HTTP/1.1 ")
                        .chars()
                        .take_while(|x| x.is_numeric())
                        .collect::<String>()
                        .parse::<u32>()
                        .unwrap_or(0);

                    if status_code != 200 {
                        continue; // ignore
                    }

                    let iter = response.iter().filter_map(|l| {
                        let mut split = l.splitn(2, ':');
                        match (split.next(), split.next()) {
                            (Some(header), Some(value)) => Some((header, value.trim())),
                            _ => None,
                        }
                    });
                    let mut dev_url: String = String::new();
                    let mut oh_device = false;
                    let mut av_device = false;
                    for (header, value) in iter {
                        if header.to_ascii_uppercase() == "LOCATION" {
                            dev_url = value.to_string();
                        } else if header.to_ascii_uppercase() == "ST" {
                            if value.contains("urn:schemas-upnp-org:service:RenderingControl:1") {
                                av_device = true;
                            } else if value.contains("urn:av-openhome-org:service:Product:1") {
                                oh_device = true;
                            }
                        }
                    }
                    if oh_device {
                        oh_devices.push((dev_url.clone(), from));
                        debug!("SSDP Discovery: OH renderer: {}", dev_url);
                    }
                    if av_device {
                        av_devices.push((dev_url.clone(), from));
                        debug!("SSDP Discovery: AV renderer: {}", dev_url);
                    }
                }
            }
            Err(e) => {
                // ignore socket read timeout on Windows or EAGAIN on Linux
                if !(e.to_string().contains("10060") || e.to_string().contains("os error 11")) {
                    logger(format!("*E*E>Error reading SSDP M-SEARCH response: {e}"));
                }
            }
        }
    }

    // only keep OH devices and AV devices that are not OH capable
    let mut usable_devices: Vec<(String, SocketAddr)> = Vec::new();
    for (oh_url, sa) in oh_devices.iter() {
        usable_devices.push((oh_url.to_string(), *sa));
    }
    for (av_url, sa) in av_devices.iter() {
        if !usable_devices.iter().any(|d| d.0 == *av_url) {
            usable_devices.push((av_url.to_string(), *sa));
        } else {
            debug!(
                "SSDP Discovery: skipping AV renderer {} as it is also OH",
                av_url
            );
        }
    }
    // now filter out devices we already know about
    for (url, sa) in usable_devices.iter() {
        if !rmap.iter().any(|m| url.contains(&m.1.dev_url)) {
            info!("SSDP discovery: new Renderer found at : {}", url);
            devices.push((url.to_string(), *sa));
        } else {
            info!("SSDP discovery: Skipping known Renderer at {}", url);
        }
    }

    // now get the new renderers description xml
    debug!("Getting new renderer descriptions");
    let mut renderers: Vec<Renderer> = Vec::new();

    for (dev, from) in devices {
        if let Some(xml) = get_service_description(&dev) {
            if let Some(mut rend) = get_renderer(&xml) {
                let mut s = from.to_string();
                if let Some(i) = s.find(':') {
                    s.truncate(i);
                }
                rend.remote_addr = s;
                // check for an absent URLBase in the description
                // or devices like Yamaha WXAD-10 with bad URLBase port number
                if rend.dev_url.is_empty() || !dev.contains(&rend.dev_url) {
                    let mut url_base = dev;
                    if url_base.contains("http://") {
                        url_base = url_base["http://".to_string().len()..].to_string();
                        let pos = url_base.find('/').unwrap_or_default();
                        if pos > 0 {
                            url_base = url_base[0..pos].to_string();
                        }
                    }
                    rend.dev_url = format!("http://{url_base}/");
                }
                renderers.push(rend);
            }
        }
    }

    for r in renderers.iter() {
        debug!(
            "Renderer {} {} ip {} at urlbase {} has {} services",
            r.dev_name,
            r.dev_model,
            r.remote_addr,
            r.dev_url,
            r.services.len()
        );
        debug!(
            "  => OpenHome Playlist control url: '{}', AvTransport url: '{}'",
            r.oh_control_url, r.av_control_url
        );
        for s in r.services.iter() {
            debug!(".. {} {} {}", s.service_type, s.service_id, s.control_url);
        }
    }
    debug!("SSDP discovery complete");
    Some(renderers)
}

/// get_service_description - get the upnp service description xml for a media renderer
fn get_service_description(dev_url: &str) -> Option<String> {
    debug!("Get service description for {}", dev_url.to_string());
    let url = dev_url.to_string();
    match ureq::get(url.as_str())
        .set("User-Agent", "swyh-rs-Rust")
        .set("Content-Type", "text/xml")
        .send_string("")
    {
        Ok(resp) => {
            let descr_xml = resp.into_string().unwrap_or_default();
            debug!("Service description:");
            debug!("{}", descr_xml);
            if !descr_xml.is_empty() {
                Some(descr_xml)
            } else {
                None
            }
        }
        Err(e) => {
            error!("Error {} getting service description for {}", e, url);
            None
        }
    }
}

/// build a renderer struct by parsing the GetDescription.xml
fn get_renderer(xml: &str) -> Option<Renderer> {
    // let xmlstream = StringReader::new(xml);
    let parser = EventReader::new(xml.as_bytes());
    let mut cur_elem = String::new();
    let mut service = AvService::new();
    let mut renderer = Renderer::new();
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                cur_elem = name.local_name;
            }
            Ok(XmlEvent::EndElement { name }) => {
                let end_elem = name.local_name;
                if end_elem == "service" {
                    if service.service_id.contains("Playlist") {
                        renderer.oh_control_url = service.control_url.clone();
                        renderer.supported_protocols |= SupportedProtocols::OPENHOME;
                    } else if service.service_id.contains("AVTransport") {
                        renderer.av_control_url = service.control_url.clone();
                        renderer.supported_protocols |= SupportedProtocols::AVTRANSPORT;
                    }
                    renderer.services.push(service);
                    service = AvService::new();
                }
            }
            Ok(XmlEvent::Characters(value)) => {
                if cur_elem.contains("serviceType") {
                    service.service_type = value;
                } else if cur_elem.contains("serviceId") {
                    service.service_id = value;
                } else if cur_elem.contains("controlURL") {
                    service.control_url = value;
                    // sometimes the control url is not prefixed with a '/'
                    if !service.control_url.is_empty() && !service.control_url.starts_with('/') {
                        service.control_url.insert(0, '/');
                    }
                } else if cur_elem.contains("modelName") {
                    renderer.dev_model = value;
                } else if cur_elem.contains("friendlyName") {
                    renderer.dev_name = value;
                } else if cur_elem.contains("deviceType") {
                    renderer.dev_type = value;
                } else if cur_elem.contains("URLBase") {
                    renderer.dev_url = value;
                }
            }
            Err(e) => {
                error!("SSDP Get Renderer Description Error: {}", e);
                return None;
            }
            _ => {}
        }
    }

    Some(renderer)
}
