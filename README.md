# Home App

Welcome to the **Home App** repository! This project is a comprehensive smart home solution designed to monitor, manage, and interact with various sensors in your environment. The repository contains everything you need to get started with setting up your smart home system, including the Home API backend, Home Sensor firmware, and detailed schematics.

## Repository Structure

- [Home API](/home-api/README.md): The core backend component of the Home App. It manages the discovery, organization, and interaction with various sensors in your home. The API hosts a web interface for real-time data visualization and provides a RESTful API for integrating sensor data into other applications.

- [Home Sensor](/home-sensor/README.md): Firmware written in C++ for the ESP32-C3 microcontroller, designed to collect temperature and humidity data. The sensor hosts a web server and exposes the collected data via a RESTful API. Future updates will include support for motion detection, noise monitoring, and compatibility with ESP8266.

## Roadmap

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
