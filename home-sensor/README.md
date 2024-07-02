# Home sensor

The ESP32 based sensor for the home app. Currently it is based on the ESP32-C3 board and is hosting an API for sending environmental data to the home api.
The temperature and humidity data is being developed.

# Schematics

The schematics for the sensor can be found in the `schematics` directory. The schematics are created using [Tinkercad®](https://www.tinkercad.com/).

# Building and flashing

Before building the project you need to create a `my-secrets.h` file. The file should contain the following:

```c

#pragma once

char ssid[] = "<SSID>";
char pass[] = "<PASSWORD>";

```

To build and flash the project you need the [Arduino IDE](https://docs.arduino.cc/software/ide/#ide-v2) and the ESP32 board installed. The board can be installed using the board manager in the Arduino IDE. The board I used is `XIAO_ESP32C3`.
