#pragma once

#include "store.hpp"

class PairStore : public Store
{
public:
    char *keys[64] = {new char[64]};
    int count = 0;
    PairStore() {}
    PairStore(Preferences *preferences, int max_size) : Store(preferences, max_size, "pair")
    {
    }

    bool has_key(char *key)
    {
        for (size_t i = 0; i < count; i++)
        {
            char *k = keys[i];
            if (k != NULL && strcmp(k, key) == 0)
            {
                return true;
            }
        }
        return false;
    }

    void init_json() override
    {
        JsonArray json_keys = doc["keys"];
        if (json_keys != NULL)
        {
            int i = 0;
            for (JsonVariant item : json_keys)
            {
                const char *key = item;
                keys[i] = new char[64];
                keys[i][0] = '\0';
                if (key == NULL)
                {
                    continue;
                }
                strcpy(keys[i], key);
                i++;
            }
            count = i;
        }
    }
    JsonDocument *as_json() override
    {
        JsonDocument new_doc;
        JsonDocument arr_doc;
        for (char *key : keys)
        {
            if (key != NULL)
            {
                arr_doc.add(key);
            }
        }
        new_doc["keys"] = arr_doc;
        doc = new_doc;

        return &doc;
    }
};