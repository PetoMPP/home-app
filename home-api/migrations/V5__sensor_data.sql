CREATE TABLE "data_schedule" (
    "features" UINT NOT NULL,
    "interval_ms" UINT NOT NULL
);

CREATE TABLE "sensor_temp_data" (
    "host" TEXT NOT NULL,
    "timestamp" UINT NOT NULL,
    "temperature" FLOAT NOT NULL,
    "humidity" FLOAT NOT NULL,
    UNIQUE("host", "timestamp")
);