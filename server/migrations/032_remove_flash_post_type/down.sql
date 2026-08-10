-- Restore the pre-removal enum values (Flash 3 and Swf 8 become gaps again).
-- Deleted Flash posts cannot be restored.

UPDATE "post" SET "type" = "type" + 1 WHERE "type" >= 3;
UPDATE "post" SET "mime_type" = "mime_type" + 1 WHERE "mime_type" >= 8;
