#pragma once

#include <WiFi.h>

#include "service.hpp"

class WifiService : public ServiceBase
{
private:
    char *ssid;
    char *pass;
    void printWifiStatus()
    {
        Serial.print("SSID: ");
        Serial.println(WiFi.SSID());

        IPAddress ip = WiFi.localIP();
        Serial.print("IP Address: ");
        Serial.println(ip);

        long rssi = WiFi.RSSI();
        Serial.print("signal strength (RSSI):");
        Serial.print(rssi);
        Serial.println(" dBm");
    }
    void connect()
    {
        Serial.print("Attempting to connect to SSID: ");
        Serial.println(ssid);

        WiFi.useStaticBuffers(true);
        WiFi.mode(WIFI_STA);
        WiFi.begin(ssid, pass);
        while (WiFi.status() != WL_CONNECTED)
        {
            delay(500);
            Serial.print(".");
        }

        Serial.println("");
        Serial.println("Connected to WiFi");
        printWifiStatus();
    }

protected:
    void handle_inner(ulong *start_ms) override
    {
    }

public:
    WifiService(char ssid[], char pass[])
    {
        this->ssid = ssid;
        this->pass = pass;
    }
    void init() override
    {
        connect();
    }
};