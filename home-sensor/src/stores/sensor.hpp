#pragma once

#include "store.hpp"

class SensorStore : public Store
{
public:
    char *name = new char[64];
    uint32_t *features = new uint32_t();
    SensorStore() {}
    SensorStore(Preferences *preferences, int max_size) : Store(preferences, max_size, "data")
    {
    }
    void init_json() override
    {
        name[0] = '\0';
        *features = 0;
        const char *n = doc["name"];
        if (n != NULL)
        {
            strcpy(name, n);
        }
        if (doc.containsKey("features"))
        {
            *features = doc["features"];
        }
    }
    JsonDocument *as_json() override
    {
        JsonDocument json;
        if (name != NULL)
        {
            json["name"] = name;
        }
        if (features != NULL)
        {
            json["features"] = *features;
        }
        else
        {
            json["features"] = nullptr;
        }
        doc = json;

        return &doc;
    }
};