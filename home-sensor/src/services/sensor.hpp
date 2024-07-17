#pragma once

#include <Preferences.h>
#include "service.hpp"
#include "../stores/sensor.hpp"

#define DATA_STORE_SIZE 0x1000

class SensorService : public ServiceBase
{
private:
    Preferences *prefs;
    SensorStore *store;

public:
    SensorService(Preferences *p)
    {
        prefs = p;
    }

    SensorStore *get_store()
    {
        store = new SensorStore(prefs, DATA_STORE_SIZE, "data");
        store->init_json();
        return store;
    }

    void save_store()
    {
        store->save(prefs, DATA_STORE_SIZE, "data");
    }

    void init() override
    {
    }

    void handle() override
    {
    }
};