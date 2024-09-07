#pragma once

#include <Preferences.h>
#include "service.hpp"
#include "../stores/sensor.hpp"

#define DATA_STORE_SIZE 0x1000

class SensorService : public ServiceBase
{
private:
    Preferences *prefs;
protected:
    void handle_inner(ulong* start_ms) override
    {
    }

public:
    SensorStore *store;
    SensorService(Preferences *p)
    {
        prefs = p;
    }

    void save_store()
    {
        store->save(prefs);
    }

    void init() override
    {
        store = new SensorStore(prefs, DATA_STORE_SIZE);
        store->init_json();
    }
};