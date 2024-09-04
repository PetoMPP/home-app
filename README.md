# Home app

My home IOT solution for managing various devices.


# Projects

## Home api

Home API is the central component of the Home App, enabling seamless discovery, management, and interaction with home sensors. It provides a web interface for monitoring real-time temperature and humidity data through interactive charts, with sensors assigned to specific areas of your home. To learn more about the project, see the [README](home-api/README.md).

## Home sensor

The Home Sensor project uses an ESP32-C3 microcontroller to monitor temperature and humidity, hosting a web server that provides data via a RESTful API. Future updates will add motion and noise detection, along with support for the ESP8266. To learn more about the project, see the [README](home-sensor/README.md).

# Roadmap

- [x] Make a web interface for managing supported devices
- [x] Add feature support for devices to mark them as able to do various actions, like:
    - [x] Manage device info
    - [x] Collect temperature/humidity data
    - [ ] Detect motion
    - [ ] Detect noises
    - [ ] Control AC units
    - [ ] Control switches/lights
    - [ ] Provide audio/video feed
- [x] Create a building based system for organizing devices and their interactions/outputs
