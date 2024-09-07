#pragma once

#include <ArduinoJson.h>
#include <Preferences.h>

class Store
{
protected:
    char *buff;
    JsonDocument doc;
    const char* name;

public:
    int len; // Valid directly after load/save operations
    int max_size;
    Store() {}
    Store(Preferences *preferences, int max_size, const char *store_name)
    {
        buff = new char[max_size];
        this->max_size = max_size;
        name = store_name;
        preferences->begin(name);
        len = preferences->getString("store", buff, max_size);
        preferences->end();
        DeserializationError err = deserializeJson(doc, buff, len);
    }
    virtual void init_json() = 0;
    virtual JsonDocument *as_json() = 0;
    virtual void load_json(JsonDocument src_doc, bool merge = false)
    {
        if (!merge)
        {
            doc = src_doc;
            return;
        }

        JsonObject::iterator iter = src_doc.as<JsonObject>().begin();
        while (iter != src_doc.as<JsonObject>().end())
        {
            if (doc.containsKey(iter->key()))
            {
                doc[iter->key()] = iter->value();
            }
            ++iter;
        }
    }
    void save(Preferences *preferences)
    {
        len = serializeJson(doc, buff, max_size);
        buff[len] = '\0';
        preferences->begin(name);
        preferences->putString("store", buff);
        preferences->end();
    }
};