# Home app

My home IOT solution for managing various devices.


# Projects

## Home api

The api for the home app. It is a server that provides a web interface for managing devices and to compose them into systems.
The backend is written in Rust using the Axum framework and the frontend is written in HTMX and templated with Askama.
To learn more about the project, see the [README](home-api/README.md).

## Home sensor

The ESP32 based sensor for the home app. It is a device that can measure various environmental data and send it to the home api.
The firmware is written in Rust using the ESP-HAL abstraction layer.
To learn more about the project, see the [README](home-sensor/README.md).

## Home common

Common code shared between the home api and home sensor projects.


# Roadmap

- [x] Make a web interface for managing supported devices
- [ ] Add feature support for devices to mark them as able to do various actions, like:
    - [ ] Manage device info
    - [ ] Collect temperature/humidity data
    - [ ] Detect motion
    - [ ] Detect noises
    - [ ] Control switches/lights
    - [ ] Provide audio/video feed
- [ ] Create a building based system for organizing devices and their interactions/outputs
