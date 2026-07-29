CREATE TABLE "post_custom_field" (
    "post_id" BIGINT NOT NULL REFERENCES "post"("id") ON DELETE CASCADE,
    "custom_field_id" BIGINT NOT NULL REFERENCES "custom_field"("id") ON DELETE CASCADE,
    "value" BIGINT NOT NULL,
    PRIMARY KEY ("post_id", "custom_field_id")
);
