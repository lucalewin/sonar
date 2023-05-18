# Sonar

Sonar allows you to use your Sonos speakers as an output for your PC audio. This project is inspired by [swyh-rs](https://github.com/dheijl/swyh-rs).

## Features

### Available features

Please note that this project is currently in early development and may be unstable.

- Stream audio from your computer to your Sonos speakers

### Planned features

- Synchronize the volume of your speakers with the volume of your PC audio
- Change the source/device of the audio stream

## Roadmap

Create a kernel-driver that uses smaller buffer sizes to reduce latency.

## Known issues

### Audio latency

There is an initial delay of approximately 500ms when starting the audio stream. After about 30 minutes, the audio and video become perfectly synced. However, after an additional 5 minutes, the audio may begin to stutter. I am actively working on a solution to this issue.

### Audio source cannot be changed

Currently, Sonar intercepts the audio stream from your default audio output device (likely your speakers or headset). In future updates, I plan to add the ability to change the audio source.

### Connecting to a speaker

Sonar does not directly connect to your speaker. Instead, the speaker makes an HTTP request to Sonar, which then sends the audio stream as a response. This means that you must manually "tell" your Sonos speaker to connect to Sonar. At present, this is not possible within Sonar itself (though I plan to add this feature in the future), so you will need to use [swyh-rs](https://github.com/dheijl/swyh-rs) to connect to your speaker initially. After that, you can simply press play on your speaker and it will connect to Sonar.
