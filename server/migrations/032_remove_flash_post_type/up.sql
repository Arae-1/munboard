-- Remove Flash (SWF) support.
--
-- PostType and MimeType are stored as SMALLINTs. Flash was PostType 3 and Swf
-- was MimeType 8. Both variants are removed from the Rust enums, which compacts
-- the remaining values, so existing rows must be remapped:
--   PostType:  Document 4 -> 3
--   MimeType:  Avif 9 -> 8, Pdf 10 -> 9
--
-- Flash posts cannot be represented anymore, so they are deleted first.

DELETE FROM "post" WHERE "type" = 3 OR "mime_type" = 8;

UPDATE "post" SET "type" = "type" - 1 WHERE "type" > 3;
UPDATE "post" SET "mime_type" = "mime_type" - 1 WHERE "mime_type" > 8;
