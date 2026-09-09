-- `schedules.id` started life as TEXT holding UUID strings. The API now models
-- it as a real `uuid`, so store it as one: Postgres rejects malformed ids for
-- us, and tokio_postgres can map the column straight onto `uuid::Uuid`.
--
-- The USING cast fails loudly if any existing row holds a non-UUID string,
-- which is the behaviour we want — better than silently dropping rows.
ALTER TABLE schedules
    ALTER COLUMN id TYPE UUID USING id::UUID,
    ALTER COLUMN id SET DEFAULT gen_random_uuid();
