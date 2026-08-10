ALTER TABLE "post_custom_field" ADD CONSTRAINT "post_custom_field_value_non_negative" CHECK ("value" >= 0);
