#pragma once

#include <ArduinoJson.h>
#include <Preferences.h>

#define STORE_SIZE 0x5000

Preferences preferences;

struct DataStore {
  char* name;
  char* location;
  uint32_t _features;
  uint32_t* features;
};

JsonDocument store_to_json(struct DataStore store) {
  JsonDocument doc;
  doc["name"] = store.name;
  doc["location"] = store.location;
  if (store.features != NULL) {
    doc["features"] = store._features;
  }

  return doc;
}

struct DataStore json_to_store(JsonDocument doc) {
  struct DataStore store = { NULL, NULL, 0, NULL};

  const char* name = doc["name"];
  if (name != NULL) {
    store.name = new char[64];
    strcpy(store.name, name);
  }
  const char* location = doc["location"];
  if (location != NULL) {
    store.location = new char[64];
    strcpy(store.location, location);
  }

  if (doc.containsKey("features")) {
    store._features = doc["features"];
    store.features = &store._features;
  }

  return store;
}

void merge_stores(DataStore* store, DataStore* rhs) {
  if (rhs->name != NULL) {
    // Serial.println(rhs->name);
    store->name = new char[64];
    strcpy(store->name, rhs->name);
  }
  if (rhs->location != NULL) {
    // Serial.println(rhs->location);
    store->location = new char[64];
    strcpy(store->location, rhs->location);
  }
  if (rhs->features != NULL) {
    // Serial.println(*rhs->features);
    store->_features = rhs->_features;
    store->features = &store->_features;  // probably not needed
  }
}

void set_data_store(struct DataStore store) {
  char* buff = new char[STORE_SIZE];
  int len = serializeJson(store_to_json(store), buff, STORE_SIZE);
  buff[len] = '\0';
  preferences.begin("data");
  preferences.putString("store", buff);
  preferences.end();
}

struct DataStore
get_data_store() {
  char* buff = new char[STORE_SIZE];
  preferences.begin("data");
  int len = preferences.getString("store", buff, STORE_SIZE);
  preferences.end();
  JsonDocument doc;
  DeserializationError err = deserializeJson(doc, buff, len);
  if (err) {
    // Fix store with empty json
    Serial.println(F("Unable to read store"));
    struct DataStore result = { new char[64], new char[64], 0 };
    set_data_store(result);
    return result;
  }

  return json_to_store(doc);
}