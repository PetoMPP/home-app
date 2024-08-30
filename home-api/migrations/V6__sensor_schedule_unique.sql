DROP TABLE IF EXISTS "data_schedule";

CREATE TABLE "data_schedule" (
    "features" UINT NOT NULL,
    "interval_ms" UINT NOT NULL,
    UNIQUE("features")
);
