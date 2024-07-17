#pragma once

#include <ArduinoJson.h>
#include <Preferences.h>

class Store
{
    protected:
    JsonDocument doc;
public:
    int len; // Valid directly after load/save operations
    int max_size;
    Store() {}
    Store(Preferences *preferences, int max_size, const char *store_name)
    {
        this->max_size = max_size;
        char *buff = new char[max_size];
        preferences->begin(store_name);
        len = preferences->getString("store", buff, max_size);
        preferences->end();
        DeserializationError err = deserializeJson(doc, buff, len);
    }
    virtual void init_json() = 0;
    virtual JsonDocument* as_json() = 0;
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
    void save(Preferences *preferences, int max_size, const char *store_name)
    {
        char *buff = new char[max_size];
        len = serializeJson(doc, buff, max_size);
        buff[len] = '\0';
        preferences->begin(store_name);
        preferences->putString("store", buff);
        preferences->end();
    }
};