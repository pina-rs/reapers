BEGIN;

--
-- ACTION CREATE TABLE
--
CREATE TABLE "player_record" (
    "id" bigserial PRIMARY KEY,
    "authUserId" uuid NOT NULL,
    "state" json NOT NULL,
    "createdAt" timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE UNIQUE INDEX "player_record__authUserId__unique_idx" ON "player_record" USING btree ("authUserId");


--
-- MIGRATION VERSION FOR reapers
--
INSERT INTO "serverpod_migrations" ("module", "version", "timestamp")
    VALUES ('reapers', '20261001123314144-player-record', now())
    ON CONFLICT ("module")
    DO UPDATE SET "version" = '20261001123314144-player-record', "timestamp" = now();

--
-- MIGRATION VERSION FOR serverpod
--
INSERT INTO "serverpod_migrations" ("module", "version", "timestamp")
    VALUES ('serverpod', '20260824182259319', now())
    ON CONFLICT ("module")
    DO UPDATE SET "version" = '20260824182259319', "timestamp" = now();

--
-- MIGRATION VERSION FOR serverpod_auth_idp
--
INSERT INTO "serverpod_migrations" ("module", "version", "timestamp")
    VALUES ('serverpod_auth_idp', '20260924105404509', now())
    ON CONFLICT ("module")
    DO UPDATE SET "version" = '20260924105404509', "timestamp" = now();

--
-- MIGRATION VERSION FOR serverpod_auth_core
--
INSERT INTO "serverpod_migrations" ("module", "version", "timestamp")
    VALUES ('serverpod_auth_core', '20260924105232991', now())
    ON CONFLICT ("module")
    DO UPDATE SET "version" = '20260924105232991', "timestamp" = now();


COMMIT;
