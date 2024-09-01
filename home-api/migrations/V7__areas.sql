CREATE TABLE "areas" (
    "rowid" INTEGER PRIMARY KEY,
    "name" TEXT NOT NULL
);

DROP TABLE IF EXISTS "sensors";

CREATE TABLE "sensors" (
    "name" TEXT NOT NULL,
    "area_id" INTEGER NULL ,
    "features" UINT NOT NULL,
    "host" TEXT PRIMARY KEY,
    "pair_id" TEXT NOT NULL,
    FOREIGN KEY ("area_id") REFERENCES "areas" ("rowid") ON DELETE SET NULL
);