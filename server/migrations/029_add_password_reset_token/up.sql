ALTER TABLE "user" ADD COLUMN "password_reset_token" TEXT;
ALTER TABLE "user" ADD COLUMN "password_reset_expires" TIMESTAMPTZ;
