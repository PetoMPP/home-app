#pragma once

#include <Preferences.h>
#include <UUID.h>
#include "service.hpp"
#include "../stores/pair.hpp"
#include "../http.hpp"

#define PAIR_BUTTON_PIN 2
#define RNG_PIN 3 // disconnected pin
#define PAIR_STORE_SIZE 0x1000
#define PAIR_TIMEOUT_MS 30000;

class PairingService : public ServiceBase
{
private:
    Preferences *prefs;
    int last_button_state;
    time_t next_pairing_expiration = 0;
    PairStore *temp_pair_store = new PairStore();
    UUID next_id;

protected:
    void handle_inner(ulong* start_ms) override
    {
        int button_state = digitalRead(PAIR_BUTTON_PIN);
        bool just_released = button_state != last_button_state && button_state == LOW;
        if (just_released)
        {
            Serial.println("Click!");
            next_pairing_expiration = *start_ms + PAIR_TIMEOUT_MS;
            pairing = true;
        }

        if (pairing && next_pairing_expiration <= *start_ms)
        {
            pairing = false;
            temp_pair_store = new PairStore();
        }

        last_button_state = button_state;
    }
public:
    PairStore *store;
    inline static const char *ERROR_MESSAGE = "To connect use /pair endpoint and pairing button on the device.";

    bool pairing = false;
    PairingService(Preferences *prefs)
    {
        this->prefs = prefs;
    }

    bool is_paired(Request *req)
    {
        char pk[64] = {0};
        get_header_value(req, "X-Pair-Id", pk);
        return pk != NULL && store->has_key(pk);
    }

    const char *generate()
    {
        next_id.generate();
        const char *id = next_id.toCharArray();
        temp_pair_store->keys[temp_pair_store->count] = new char[64];
        strcpy(temp_pair_store->keys[temp_pair_store->count], id);
        temp_pair_store->count++;
        return id;
    }

    bool pair(Request *req)
    {
        char key[64] = {0};
        get_header_value(req, "X-Pair-Id", key);
        if (key == NULL || !temp_pair_store->has_key(key))
        {
            return false;
        }

        store->keys[store->count] = new char[64];
        strcpy(store->keys[store->count], key);
        store->count++;
        store->as_json();
        store->save(prefs, "pair");
        return true;
    }

    void init() override
    {
        pinMode(PAIR_BUTTON_PIN, INPUT);
        last_button_state = digitalRead(PAIR_BUTTON_PIN);
        randomSeed(analogRead(RNG_PIN));
        next_id.seed(random());
        store = new PairStore(prefs, PAIR_STORE_SIZE, "pair");
        store->init_json();
    }
};