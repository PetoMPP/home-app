DROP TABLE "sensors";
CREATE TABLE "sensors" (
    "name" TEXT NOT NULL,
    "location" TEXT NOT NULL,
    "features" UINT NOT NULL,
    "host" TEXT PRIMARY KEY,
    "pair_id" TEXT NOT NULL
);