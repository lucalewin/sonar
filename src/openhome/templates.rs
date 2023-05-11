/// OH insert playlist template
pub static OH_INSERT_PL_TEMPLATE: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<s:Envelope s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\" xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\">\
<s:Body>\
<u:Insert xmlns:u=\"urn:av-openhome-org:service:Playlist:1\">\
<AfterId>0</AfterId>\
<Uri>{server_uri}</Uri>\
<Metadata>{didl_data}</Metadata>\
</u:Insert>\
</s:Body>\
</s:Envelope>";

/// AV SetTransportURI template
pub static AV_SET_TRANSPORT_URI_TEMPLATE: &str = "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">\
<s:Body>\
<u:SetAVTransportURI xmlns:u=\"urn:schemas-upnp-org:service:AVTransport:1\">\
<InstanceID>0</InstanceID>\
<CurrentURI>{server_uri}</CurrentURI>\
<CurrentURIMetaData>{didl_data}</CurrentURIMetaData>\
</u:SetAVTransportURI>\
</s:Body>\
</s:Envelope>";

/// didl protocolinfo
pub static L16_PROT_INFO: &str =
    "http-get:*:audio/L16;rate={sample_rate};channels=2:DLNA.ORG_PN=LPCM";
pub static L24_PROT_INFO: &str =
    "http-get:*:audio/L24;rate={sample_rate};channels=2:DLNA.ORG_PN=LPCM";
pub static WAV_PROT_INFO: &str = "http-get:*:audio/wav:DLNA.ORG_PN=WAV;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=03700000000000000000000000000000";
pub static FLAC_PROT_INFO: &str = "http-get:*:audio/flac:DLNA.ORG_PN=FLAC;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000";

/// didl metadata template
pub static DIDL_TEMPLATE: &str = "\
<DIDL-Lite xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\">\
<item id=\"1\" parentID=\"0\" restricted=\"0\">\
<dc:title>swyh-rs</dc:title>\
<res bitsPerSample=\"{bits_per_sample}\" \
nrAudioChannels=\"2\" \
sampleFrequency=\"{sample_rate}\" \
protocolInfo=\"{didl_prot_info}\" \
duration=\"{duration}\" >{server_uri}</res>\
<upnp:class>object.item.audioItem.musicTrack</upnp:class>\
</item>\
</DIDL-Lite>";

/// OH play playlist template
pub static OH_PLAY_PL_TEMPLATE: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<s:Envelope s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\" \
xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\">\
<s:Body>\
<u:Play xmlns:u=\"urn:av-openhome-org:service:Playlist:1\"/>\
</s:Body>\
</s:Envelope>";

/// AV Play template
pub static AV_PLAY_TEMPLATE: &str = "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<s:Envelope s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\" xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\">\
<s:Body>\
<u:Play xmlns:u=\"urn:schemas-upnp-org:service:AVTransport:1\">\
<InstanceID>0</InstanceID>\
<Speed>1</Speed>\
</u:Play>\
</s:Body>\
</s:Envelope>";

/// OH delete playlist template
pub static OH_DELETE_PL_TEMPLATE: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<s:Envelope s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\" \
xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\">\
<s:Body>\
<u:DeleteAll xmlns:u=\"urn:av-openhome-org:service:Playlist:1\"/>\
</s:Body>\
</s:Envelope>";

/// AV Stop play template
pub static AV_STOP_PLAY_TEMPLATE: &str ="\
<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<s:Envelope s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\" xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\">\
<s:Body>\
<u:Stop xmlns:u=\"urn:schemas-upnp-org:service:AVTransport:1\">\
<InstanceID>0</InstanceID>\
</u:Stop>\
</s:Body>\
</s:Envelope>";

// SSDP UDP search message for media renderers with a 3.0 second MX response time
pub static SSDP_DISCOVER_MSG: &str = "M-SEARCH * HTTP/1.1\r\n\
Host: 239.255.255.250:1900\r\n\
Man: \"ssdp:discover\"\r\n\
ST: {device_type}\r\n\
MX: 3\r\n\r\n";
