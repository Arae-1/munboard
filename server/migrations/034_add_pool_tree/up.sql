CREATE TABLE "pool_tree_lock" (
    "id" BOOLEAN PRIMARY KEY DEFAULT true,
    CONSTRAINT "pool_tree_lock_is_singular" CHECK ("id")
);
INSERT INTO "pool_tree_lock" ("id") VALUES (true);

CREATE TABLE "pool_subpool" (
    "parent_id" BIGINT NOT NULL REFERENCES "pool" ON DELETE RESTRICT,
    "child_id" BIGINT NOT NULL REFERENCES "pool" ON DELETE RESTRICT UNIQUE,
    "order" BIGINT NOT NULL,
    PRIMARY KEY ("parent_id", "child_id"),
    CHECK ("parent_id" <> "child_id")
);
CREATE INDEX "idx_pool_subpool_parent_order" ON "pool_subpool" ("parent_id", "order");

CREATE FUNCTION verify_pool_subpool() RETURNS TRIGGER AS $$
BEGIN
    -- One row locks every hierarchy change. This makes cycle validation serial.
    PERFORM 1 FROM "pool_tree_lock" WHERE "id" FOR UPDATE;

    IF NEW."parent_id" = NEW."child_id" THEN
        RAISE EXCEPTION 'a pool cannot be its own parent' USING ERRCODE = '23514';
    END IF;

    IF EXISTS (
        WITH RECURSIVE descendants("id") AS (
            SELECT "child_id" FROM "pool_subpool" WHERE "parent_id" = NEW."child_id"
            UNION
            SELECT edge."child_id"
            FROM "pool_subpool" AS edge
            JOIN descendants ON edge."parent_id" = descendants."id"
        )
        SELECT 1 FROM descendants WHERE "id" = NEW."parent_id"
    ) THEN
        RAISE EXCEPTION 'a pool tree cannot contain a cycle' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER verify_pool_subpool_trigger
BEFORE INSERT OR UPDATE ON "pool_subpool"
FOR EACH ROW EXECUTE FUNCTION verify_pool_subpool();
