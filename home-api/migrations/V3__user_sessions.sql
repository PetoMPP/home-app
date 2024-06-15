CREATE TABLE "user_sessions" (
    "normalized_name" TEXT NOT NULL,
    "token" TEXT PRIMARY KEY,
    FOREIGN KEY ("normalized_name") REFERENCES "users" ("normalized_name")
);