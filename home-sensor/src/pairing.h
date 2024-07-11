#pragma once

#include <ArduinoJson.h>
#include <Preferences.h>
#include <UUID.h>

Preferences preferences;

#define PAIR_BUTTON_PIN 2
#define RNG_PIN 3 // disconnected pin
#define PAIR_STORE_SIZE 0x1000
#define PAIR_TIMEOUT_MS 30000;

int last_button_state;
ulong next_pairing_expiration = 0;
bool pairing = false;
UUID next_id;

void pairing_init() {
  pinMode(PAIR_BUTTON_PIN, INPUT);
  last_button_state = digitalRead(PAIR_BUTTON_PIN);
  randomSeed(analogRead(RNG_PIN));
  next_id.seed(random());
}

struct PairStore {
  char* keys[64];
  int count;
};

struct PairStore curr_pair_keys = { { "" }, 0 };

void handle_pairing() {
  int button_state = digitalRead(PAIR_BUTTON_PIN);
  bool just_released = button_state != last_button_state && button_state == LOW;
  if (just_released) {
    Serial.println("Click!");
    next_pairing_expiration = millis() + PAIR_TIMEOUT_MS;
    pairing = true;
  }

  if (pairing && next_pairing_expiration <= millis()) {
    pairing = false;
    curr_pair_keys = { { "" }, 0 };
  }

  last_button_state = button_state;
}

const char* generate_pair_id() {
  next_id.generate();
  const char* id = next_id.toCharArray();
  curr_pair_keys.keys[curr_pair_keys.count] = new char[64];
  strcpy(curr_pair_keys.keys[curr_pair_keys.count], id);
  curr_pair_keys.count++;
  return id;
}

bool has_pair_key(PairStore* store, const char* pair_key) {
  for (char* key : store->keys) {
    if (key != NULL && strcmp(pair_key, key) == 0) {
      return true;
    }
  }

  return false;
}

JsonDocument pair_store_to_json(struct PairStore store) {
  JsonDocument doc;
  JsonDocument arr_doc;
  for (char* key : store.keys) {
    if (key != NULL) {
      arr_doc.add(key);
    }
  }
  doc["keys"] = arr_doc;

  return doc;
}

struct PairStore
json_to_pair_store(JsonDocument doc) {
  struct PairStore store = { { "" }, 0 };
  JsonArray keys = doc["keys"];
  if (keys != NULL) {
    int i = 0;
    for (JsonVariant item : keys) {
      const char* key = item;
      store.keys[i] = new char[64];
      store.keys[i][0] = '\0';
      if (key == NULL) {
        continue;
      }
      strcpy(store.keys[i], key);
      i++;
    }
    store.count = i;
  }
  return store;
}

void set_pair_store(struct PairStore store, int* len) {
  char* buff = new char[PAIR_STORE_SIZE];
  *len = serializeJson(pair_store_to_json(store), buff, PAIR_STORE_SIZE);
  int last = *len;
  buff[last] = '\0';
  preferences.begin("pair");
  preferences.putString("store", buff);
  preferences.end();
}

struct PairStore get_pair_store(int* len) {
  char* buff = new char[PAIR_STORE_SIZE];
  preferences.begin("pair");
  *len = preferences.getString("store", buff, PAIR_STORE_SIZE);
  preferences.end();
  JsonDocument doc;
  DeserializationError err = deserializeJson(doc, buff, *len);
  if (err) {
    struct PairStore store = { { "" }, 0 };
    // Fix store with empty json
    Serial.println(F("Unable to read store"));
    set_pair_store(store, len);
    return store;
  }

  return json_to_pair_store(doc);
}