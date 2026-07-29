CREATE TABLE "custom_field" (
    "id" BIGSERIAL PRIMARY KEY,
    "name" CITEXT NOT NULL UNIQUE,
    "creation_time" TIMESTAMPTZ NOT NULL DEFAULT now()
);
