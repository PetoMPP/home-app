#pragma once

#include "service.hpp"

#define LED_PIN 7
#define IDLE_BLINK_HZ 0.5
#define IDLE_INTERVAL 1000 / IDLE_BLINK_HZ

class LedService : public ServiceBase
{
private:
    ulong last_blink = 0;
    bool led_state = false;

protected:
    void handle_inner(ulong* start_ms) override
    {
        if (*start_ms - last_blink > IDLE_INTERVAL)
        {
            last_blink = *start_ms;
            set(!led_state);
        }
    }

public:
    LedService() {}
    void set(bool mode)
    {
        digitalWrite(LED_PIN, mode);
        led_state = mode;
    }
    void init() override
    {
        pinMode(LED_PIN, OUTPUT);
    }
};