DROP INDEX IF EXISTS "user_token_token_hash";

ALTER TABLE "user_token" DROP COLUMN "token_secret";
ALTER TABLE "user_token" DROP COLUMN "token_hash";
