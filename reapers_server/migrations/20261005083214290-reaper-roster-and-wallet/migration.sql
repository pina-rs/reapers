BEGIN;

--
-- ACTION CREATE TABLE
--
CREATE TABLE "reaper_record" (
    "id" bigserial PRIMARY KEY,
    "assetKey" text NOT NULL,
    "collection" text NOT NULL,
    "state" json NOT NULL
);

-- Indexes
CREATE UNIQUE INDEX "reaper_record__assetKey__unique_idx" ON "reaper_record" USING btree ("assetKey");

--
-- ACTION CREATE TABLE
--
CREATE TABLE "wallet_challenge" (
    "id" bigserial PRIMARY KEY,
    "challengeId" uuid NOT NULL,
    "authUserId" uuid NOT NULL,
    "address" text NOT NULL,
    "message" text NOT NULL,
    "expiresAt" timestamp without time zone NOT NULL,
    "consumed" boolean NOT NULL DEFAULT false
);

-- Indexes
CREATE UNIQUE INDEX "wallet_challenge__challengeId__unique_idx" ON "wallet_challenge" USING btree ("challengeId");
CREATE UNIQUE INDEX "wallet_challenge__authUserId__unique_idx" ON "wallet_challenge" USING btree ("authUserId");

--
-- ACTION CREATE TABLE
--
CREATE TABLE "wallet_identity" (
    "id" bigserial PRIMARY KEY,
    "address" text NOT NULL,
    "authUserId" uuid NOT NULL
);

-- Indexes
CREATE UNIQUE INDEX "wallet_identity__address__unique_idx" ON "wallet_identity" USING btree ("address");
CREATE UNIQUE INDEX "wallet_identity__authUserId__unique_idx" ON "wallet_identity" USING btree ("authUserId");


--
-- MIGRATION VERSION FOR reapers
--
INSERT INTO "serverpod_migrations" ("module", "version", "timestamp")
    VALUES ('reapers', '20261005083214290-reaper-roster-and-wallet', now())
    ON CONFLICT ("module")
    DO UPDATE SET "version" = '20261005083214290-reaper-roster-and-wallet', "timestamp" = now();

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
