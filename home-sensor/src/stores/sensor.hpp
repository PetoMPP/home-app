#pragma once

#include "store.hpp"

class SensorStore : public Store
{
public:
    char *name = new char[64];
    uint32_t *features = new uint32_t(0b1);
    SensorStore() {}
    SensorStore(Preferences *preferences, int max_size) : Store(preferences, max_size, "data")
    {
    }
    void init_json() override
    {
        name[0] = '\0';
        const char *n = doc["name"];
        if (n != NULL)
        {
            strcpy(name, n);
        }
    }
    JsonDocument *as_json() override
    {
        JsonDocument json;
        if (name != NULL)
        {
            json["name"] = name;
        }
        json["features"] = *features;
        doc = json;

        return &doc;
    }
};