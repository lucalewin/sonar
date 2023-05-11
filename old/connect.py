import requests

WAV_PROT_INFO = "http-get:*:audio/wav:DLNA.ORG_PN=WAV;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=03700000000000000000000000000000"

AV_SET_TRANSPORT_URI_TEMPLATE = "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">\
    <s:Body>\
        <u:SetAVTransportURI xmlns:u=\"urn:schemas-upnp-org:service:AVTransport:1\">\
            <InstanceID>0</InstanceID>\
            <CurrentURI>http://192.168.178.60:8764/test.wav</CurrentURI>\
            <CurrentURIMetaData>\
                <DIDL-Lite xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\">\
                    <item id=\"1\" parentID=\"0\" restricted=\"0\">\
                    <dc:title>swyh-rs</dc:title>\
                    <res bitsPerSample=\"16\" \
                        nrAudioChannels=\"2\" \
                        sampleFrequency=\"48000\" \
                        protocolInfo=\"http-get:*:audio/wav:DLNA.ORG_PN=WAV;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=03700000000000000000000000000000\" \
                        duration=\"00:00:00\" >http://192.168.178.60:8764/test.wav</res>\
                    <upnp:class>object.item.audioItem.musicTrack</upnp:class>\
                    </item>\
                </DIDL-Lite>\
            </CurrentURIMetaData>\
        </u:SetAVTransportURI>\
    </s:Body>\
</s:Envelope>"

AV_PLAY_TEMPLATE = "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<s:Envelope s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\" xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\">\
    <s:Body>\
        <u:Play xmlns:u=\"urn:schemas-upnp-org:service:AVTransport:1\">\
            <InstanceID>0</InstanceID>\
            <Speed>1</Speed>\
        </u:Play>\
    </s:Body>\
</s:Envelope>"

headers = {
    "Connection": "close",
    "User-Agent": "sonar-connector",
    "Accept": "*/*",
    "SOAPAction": "",
    "Content-Type": "text/xml; charset:\"utf-8\"",
}

res = requests.post("http://192.168.178.61:1400/MediaRenderer/AVTransport/Control", headers=headers, data=AV_SET_TRANSPORT_URI_TEMPLATE)

print(res)
