#pragma once

#include "service.hpp"

#define LED_PIN 7

class LedService : ServiceBase
{
public:
    LedService() {}
    void set(bool mode)
    {
        digitalWrite(LED_PIN, mode);
    }
    void init() override
    {
        pinMode(LED_PIN, OUTPUT);
    }

    void handle() override
    {
    }
};