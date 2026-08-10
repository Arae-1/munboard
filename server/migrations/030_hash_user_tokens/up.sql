-- Store only a BLAKE3 hash (hex) and an AES-256-GCM ciphertext of each user
-- token; the plaintext token value is never persisted.
--
-- Existing rows are deleted: legacy rows stored the raw token in the `id`
-- column, so their hashes cannot be recomputed and the tokens are revoked.
DELETE FROM "user_token";

ALTER TABLE "user_token" ADD COLUMN "token_hash" TEXT NOT NULL DEFAULT '';
ALTER TABLE "user_token" ADD COLUMN "token_secret" BYTEA NOT NULL DEFAULT '\x';

-- Real tokens get a unique 64-char hex hash. Empty-string hashes (rows without
-- a token value, e.g. legacy inserts) may repeat.
CREATE UNIQUE INDEX "user_token_token_hash" ON "user_token" ("token_hash") WHERE "token_hash" <> '';
