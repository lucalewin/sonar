//! controller for avmedia renderers (audio only) using OpenHome protocol
//!
//! Only tested with Volumio streamers (<https://volumio.org/>)

pub mod ssdp;
mod templates;
#[cfg(test)]
mod tests;

use crate::streaming::StreamingFormat;
use bitflags::bitflags;
use log::{debug, error};
use std::{collections::HashMap, net::IpAddr, time::Duration};
use strfmt::strfmt;
use url::Url;

use templates::*;

/// Bad XML template error
static BAD_TEMPL: &str = "Bad xml template (strfmt)";

#[derive(Debug)]
pub struct StreamInfo {
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub streaming_format: StreamingFormat,
}

/// An UPNP/DLNA service desciption
#[derive(Debug, Clone)]
pub struct AvService {
    service_id: String,
    service_type: String,
    control_url: String,
}

impl AvService {
    fn new() -> AvService {
        AvService {
            service_id: String::new(),
            service_type: String::new(),
            control_url: String::new(),
        }
    }
}

bitflags! {
    /// supported UPNP/DLNA protocols
    #[derive(Debug, Clone)]
    pub struct SupportedProtocols: u32 {
        const NONE        = 0b0000;
        const OPENHOME    = 0b0001;
        const AVTRANSPORT = 0b0010;
        const ALL = Self::OPENHOME.bits() | Self::AVTRANSPORT.bits();
    }
}

/// Renderer struct describers a media renderer, info is collected from GetDescription.xml
#[derive(Debug, Clone)]
pub struct Renderer {
    pub dev_name: String,
    pub dev_model: String,
    pub dev_type: String,
    pub dev_url: String,
    pub oh_control_url: String,
    pub av_control_url: String,
    pub supported_protocols: SupportedProtocols,
    pub remote_addr: String,
    pub services: Vec<AvService>,
}

impl Renderer {
    fn new() -> Renderer {
        Renderer {
            dev_name: String::new(),
            dev_url: String::new(),
            dev_model: String::new(),
            dev_type: String::new(),
            av_control_url: String::new(),
            oh_control_url: String::new(),
            supported_protocols: SupportedProtocols::NONE,
            remote_addr: String::new(),
            services: Vec::new(),
        }
    }

    fn parse_url(&self, dev_url: &str, log: &dyn Fn(String)) -> (String, u16) {
        let host: String;
        let port: u16;
        match Url::parse(dev_url) {
            Ok(url) => {
                host = url.host_str().unwrap().to_string();
                port = url.port_or_known_default().unwrap();
            }
            Err(e) => {
                log(format!(
                    "parse_url(): Error '{e}' while parsing base url '{dev_url}'"
                ));
                host = "0.0.0.0".to_string();
                port = 0;
            }
        }
        (host, port)
    }

    /// oh_soap_request - send an OpenHome SOAP message to a renderer
    fn soap_request(&self, url: &str, soap_action: &str, body: &str) -> Option<String> {
        debug!(
            "url: {},\r\n=>SOAP Action: {},\r\n=>SOAP xml: \r\n{}",
            url.to_string(),
            soap_action,
            body
        );
        match ureq::post(url)
            .set("Connection", "close")
            .set("User-Agent", "swyh-rs-Rust/0.x")
            .set("Accept", "*/*")
            .set("SOAPAction", &format!("\"{soap_action}\""))
            .set("Content-Type", "text/xml; charset=\"utf-8\"")
            .send_string(body)
        {
            Ok(resp) => {
                let xml = resp.into_string().unwrap();
                debug!("<=SOAP response: {}\r\n", xml);
                Some(xml)
            }
            Err(e) => {
                error!("<= SOAP POST error: {}\r\n", e);
                None
            }
        }
    }

    /// play - start play on this renderer, using Openhome if present, else AvTransport (if present)
    pub fn play(
        &self,
        local_addr: &IpAddr,
        server_port: u16,
        log: &dyn Fn(String),
        streaminfo: &StreamInfo,
    ) -> Result<(), &str> {
        let use_wav_format = streaminfo.streaming_format == StreamingFormat::Wav;
        // build the hashmap with the formatting vars for the OH and AV play templates
        let mut fmt_vars = HashMap::new();
        let (host, port) = self.parse_url(&self.dev_url, log);
        let addr = format!("{local_addr}:{server_port}");
        let local_url = format!("http://{addr}/stream/swyh.wav");
        fmt_vars.insert("server_uri".to_string(), local_url);
        fmt_vars.insert(
            "bits_per_sample".to_string(),
            streaminfo.bits_per_sample.to_string(),
        );
        fmt_vars.insert(
            "sample_rate".to_string(),
            streaminfo.sample_rate.to_string(),
        );
        fmt_vars.insert("duration".to_string(), "00:00:00".to_string());
        let mut didl_prot: String;
        if streaminfo.streaming_format == StreamingFormat::Flac {
            didl_prot = htmlescape::encode_minimal(FLAC_PROT_INFO);
        } else if use_wav_format {
            didl_prot = htmlescape::encode_minimal(WAV_PROT_INFO);
        } else if streaminfo.bits_per_sample == 16 {
            didl_prot = htmlescape::encode_minimal(L16_PROT_INFO);
        } else {
            didl_prot = htmlescape::encode_minimal(L24_PROT_INFO);
        }
        match strfmt(&didl_prot, &fmt_vars) {
            Ok(s) => didl_prot = s,
            Err(e) => {
                didl_prot = format!("oh_play: error {e} formatting didl_prot");
                log(didl_prot.clone());
                return Err(BAD_TEMPL);
            }
        }
        fmt_vars.insert("didl_prot_info".to_string(), didl_prot);
        let mut didl_data = htmlescape::encode_minimal(DIDL_TEMPLATE);
        match strfmt(&didl_data, &fmt_vars) {
            Ok(s) => didl_data = s,
            Err(e) => {
                didl_data = format!("oh_play: error {e} formatting didl_data xml");
                log(didl_data.clone());
                return Err(BAD_TEMPL);
            }
        }
        fmt_vars.insert("didl_data".to_string(), didl_data);
        // now send the start playing commands
        if self
            .supported_protocols
            .contains(SupportedProtocols::OPENHOME)
        {
            log(format!(
            "OH Start playing on {} host={host} port={port} from {local_addr} using OpenHome Playlist",
            self.dev_name));
            return self.oh_play(log, &fmt_vars);
        } else if self
            .supported_protocols
            .contains(SupportedProtocols::AVTRANSPORT)
        {
            log(format!(
            "AV Start playing on {} host={host} port={port} from {local_addr} using AvTransport Play",
            self.dev_name));
            return self.av_play(log, &fmt_vars);
        } else {
            log("ERROR: play: no supported renderer protocol found".to_string());
        }
        Ok(())
    }

    /// oh_play - set up a playlist on this OpenHome renderer and tell it to play it
    ///
    /// the renderer will then try to get the audio from our built-in webserver
    /// at http://{_my_ip_}:{server_port}/stream/swyh.wav  
    fn oh_play(
        &self,
        log: &dyn Fn(String),
        fmt_vars: &HashMap<String, String>,
    ) -> Result<(), &str> {
        // stop anything currently playing first, Moode needs it
        self.oh_stop_play(log);
        // Send the InsertPlayList command with metadate(DIDL-Lite)
        let (host, port) = self.parse_url(&self.dev_url, log);
        log(format!(
            "OH Inserting new playlist on {} host={host} port={port}",
            self.dev_name
        ));
        let xmlbody = match strfmt(OH_INSERT_PL_TEMPLATE, fmt_vars) {
            Ok(s) => s,
            Err(e) => {
                log(format!("oh_play: error {e} formatting oh playlist xml"));
                return Err(BAD_TEMPL);
            }
        };
        let url = format!("http://{host}:{port}{}", self.oh_control_url);
        let _resp = self
            .soap_request(
                &url,
                "urn:av-openhome-org:service:Playlist:1#Insert",
                &xmlbody,
            )
            .unwrap_or_default();
        // send the Play command
        log(format!(
            "OH Play on {} host={host} port={port}",
            self.dev_name
        ));
        let _resp = self
            .soap_request(
                &url,
                "urn:av-openhome-org:service:Playlist:1#Play",
                OH_PLAY_PL_TEMPLATE,
            )
            .unwrap_or_default();
        Ok(())
    }

    /// av_play - send the AVTransport URI to the player and tell it to play
    ///
    /// the renderer will then try to get the audio from our built-in webserver
    /// at http://{_my_ip_}:{server_port}/stream/swyh.wav  
    fn av_play(
        &self,
        log: &dyn Fn(String),
        fmt_vars: &HashMap<String, String>,
    ) -> Result<(), &str> {
        // to prevent error 705 (transport locked) on some devices
        // it's necessary to send a stop play request first
        self.av_stop_play(log);
        // now send SetAVTransportURI with metadate(DIDL-Lite) and play requests
        let xmlbody = match strfmt(AV_SET_TRANSPORT_URI_TEMPLATE, fmt_vars) {
            Ok(s) => s,
            Err(e) => {
                log(format!("av_play: error {e} formatting set transport uri"));
                return Err(BAD_TEMPL);
            }
        };
        let (host, port) = self.parse_url(&self.dev_url, log);
        let url = format!("http://{host}:{port}{}", self.av_control_url);
        let _resp = self
            .soap_request(
                &url,
                "urn:schemas-upnp-org:service:AVTransport:1#SetAVTransportURI",
                &xmlbody,
            )
            .unwrap_or_default();
        // the renderer will now send a head request first, so wait a bit
        std::thread::sleep(Duration::from_millis(100));
        // send play command
        let _resp = self
            .soap_request(
                &url,
                "urn:schemas-upnp-org:service:AVTransport:1#Play",
                AV_PLAY_TEMPLATE,
            )
            .unwrap_or_default();
        Ok(())
    }

    /// stop_play - stop playing on this renderer (OpenHome or AvTransport)
    pub fn stop_play(&self, log: &dyn Fn(String)) {
        if self
            .supported_protocols
            .contains(SupportedProtocols::OPENHOME)
        {
            self.oh_stop_play(log)
        } else if self
            .supported_protocols
            .contains(SupportedProtocols::AVTRANSPORT)
        {
            self.av_stop_play(log)
        } else {
            log("ERROR: stop_play: no supported renderer protocol found".to_string());
        }
    }

    /// oh_stop_play - delete the playlist on the OpenHome renderer, so that it stops playing
    fn oh_stop_play(&self, log: &dyn Fn(String)) {
        let (host, port) = self.parse_url(&self.dev_url, log);
        let url = format!("http://{host}:{port}{}", self.oh_control_url);
        log(format!(
            "OH Deleting current playlist on {} host={host} port={port}",
            self.dev_name
        ));

        // delete current playlist
        let _resp = self
            .soap_request(
                &url,
                "urn:av-openhome-org:service:Playlist:1#DeleteAll",
                OH_DELETE_PL_TEMPLATE,
            )
            .unwrap_or_default();
    }

    /// av_stop_play - stop playing on the AV renderer
    fn av_stop_play(&self, log: &dyn Fn(String)) {
        let (host, port) = self.parse_url(&self.dev_url, log);
        let url = format!("http://{host}:{port}{}", self.av_control_url);
        log(format!(
            "AV Stop playing on {} host={host} port={port}",
            self.dev_name
        ));

        // delete current playlist
        let _resp = self
            .soap_request(
                &url,
                "urn:schemas-upnp-org:service:AVTransport:1#Stop",
                AV_STOP_PLAY_TEMPLATE,
            )
            .unwrap_or_default();
    }
}
