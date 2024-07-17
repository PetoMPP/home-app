# Home sensor

The ESP32 based sensor for the home app. Currently it is based on the ESP32-C3 board and is hosting an API for sending environmental data to the home api.
The temperature and humidity data is being developed.

# Schematics

The schematics for the sensor can be found in the `schematics` directory. The schematics are created using [Tinkercad®](https://www.tinkercad.com/).

# Building and flashing

Before building the project you need to create a `secrets.h` file in src directory. The file should contain the following:

```c

#pragma once

char ssid[] = "<SSID>";
char pass[] = "<PASSWORD>";

```

To build and flash the project you need the [Arduino VS Code Extension](https://marketplace.visualstudio.com/items?itemName=vsciot-vscode.vscode-arduino). Configuration file `.vscode/arduino.json` is provided with optimal setting for `XIAO_ESP32C3` board and only `port` might need adjustments. 