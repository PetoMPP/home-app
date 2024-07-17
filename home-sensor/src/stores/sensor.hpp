#pragma once

#include "store.hpp"

class SensorStore : public Store
{
public:
    char *name = new char[64];
    char *location = new char[64];
    uint32_t *features;
    SensorStore() {}
    SensorStore(Preferences *preferences, int max_size, const char *store_name) : Store(preferences, max_size, store_name)
    {
    }
    void init_json() override
    {
        const char *n = doc["name"];
        if (n != NULL)
        {
            name = new char[64];
            strcpy(name, n);
        }
        const char *l = doc["location"];
        if (l != NULL)
        {
            location = new char[64];
            strcpy(location, l);
        }

        if (doc.containsKey("features"))
        {
            features = new uint32_t(doc["features"]);
        }
    }
    JsonDocument *as_json() override
    {
        JsonDocument json;
        if (name != NULL)
        {
            json["name"] = name;
        }
        if (location != NULL)
        {
            json["location"] = location;
        }
        if (features != NULL)
        {
            json["features"] = *features;
        }
        doc = json;

        return &doc;
    }
};