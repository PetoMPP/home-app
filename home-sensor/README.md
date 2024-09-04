# Home Sensor

The **Home Sensor** project is designed to collect and expose environmental data using an ESP32-C3 microcontroller. This project currently focuses on temperature and humidity monitoring, with plans to expand functionality in the future. The sensor hosts a web server that provides access to collected data through a simple API, making it an integral part of your smart home system.

## Current Features

- **Temperature and Humidity Monitoring**: The Home Sensor continuously collects temperature and humidity data, making it available via a built-in web server.

- **Web Server with API**: Data is exposed through a RESTful API, allowing for easy integration with other smart home components and applications.

## Future Features

- **Motion Detection**: Support for motion sensors will be added, enabling the Home Sensor to detect movement and integrate with home security systems.

- **Noise Detection**: Future updates will include noise detection, allowing the sensor to monitor sound levels in your environment.

- **ESP8266 Support**: In addition to the ESP32-C3, support for the ESP8266 microcontroller will be introduced, broadening the hardware compatibility of the project.

## Schematics

Schematics for the Home Sensor project are available in the `schematics` directory. These schematics were created using TinkerCad and provide a detailed guide on how to wire the components for the ESP32-C3.

## Building and Flashing

To build and flash the Home Sensor firmware, you can use the Arduino VSCode extension. Follow these steps to get started:

1. **Install Prerequisites**:

   - Install the [Arduino VSCode extension](https://marketplace.visualstudio.com/items?itemName=vsciot-vscode.vscode-arduino).
   - Ensure you have the necessary libraries and board support for the ESP32-C3.

2. **Open the Project**:

   - Open the Home Sensor project in VSCode.

3. **Create secret.h file**:

   - Before building the project you need to create a `secret.h` file in src directory. The file should contain the following:
    ```c

    #pragma once

    char ssid[] = "<SSID>";
    char pass[] = "<PASSWORD>";

    ```

4. **Build and Flash**:

   - Use the Arduino VSCode extension to build the project.
   - After the build is complete, you can also use the Arduino VSCode extension to flash the firmware directly to your ESP32-C3 board.

## Getting Started

After flashing the firmware, the Home Sensor will start collecting data and hosting the web server. You can access the API through your local network to retrieve temperature and humidity data.

This project is designed to be easily expandable, with additional sensors and features planned for future updates. Stay tuned for new releases that will bring even more functionality to your Home Sensor.
