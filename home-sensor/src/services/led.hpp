#pragma once

#include "service.hpp"

#define LED_PIN 7
#define IDLE_BLINK_HZ 1.11
#define IDLE_INTERVAL 1000 / IDLE_BLINK_HZ

class LedService : public ServiceBase
{
private:
    ulong last_blink = 0;
    bool led_state = false;

protected:
    void handle_inner(ulong* start_ms) override
    {
        if (!blinking && !led_state)
        {
            return;
        }

        if (*start_ms - last_blink > IDLE_INTERVAL)
        {
            last_blink = *start_ms;
            set(!led_state);
        }
    }

public:
    bool blinking = false;
    LedService() {}
    void set(bool val)
    {
        digitalWrite(LED_PIN, val);
        led_state = val;
    }
    void init() override
    {
        pinMode(LED_PIN, OUTPUT);
    }
};