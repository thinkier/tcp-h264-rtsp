# `tcp+h264://` to `rtsp://` transcoder server

## Use Case / Target
`raspivid` and `libcamera-vid` both have `TCP/h264` socket support, and therefore we can offload the RTSP server to a beefier machine instead of invoking `cvlc` to start an RTSP server on the Pi.

## Goals

### Primary Goal

- [x] To allow RTSP streaming video on demand. Without hiccups / lags / any inconveniences from running the RTSP server on a Pi.
- [x] To allow multiple RTSP streams on the same application server

### Stretch Goals

- [ ] To pause the RTSP server when no streams are requested to minimize resource usage.
- [ ] To modify the raw h264 stream with OpenCV or similar applications
