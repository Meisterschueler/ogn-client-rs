-- table creation script for PostgreSQL (with PostGIS extension)

CREATE TABLE IF NOT EXISTS errors (
    "ts"                TIMESTAMPTZ NOT NULL,

    raw_message         TEXT,
    error_message       TEXT
);

CREATE TABLE IF NOT EXISTS server_comments (
    "ts"                TIMESTAMPTZ NOT NULL,

    version             TEXT,
    server_ts           TIMESTAMPTZ,
    server              TEXT,
    ip_address          TEXT,
    port                INTEGER
);

CREATE TABLE IF NOT EXISTS positions (
    "ts"                TIMESTAMPTZ NOT NULL,

    -- APRS message body
    src_call            VARCHAR(9) NOT NULL,
    dst_call            VARCHAR(9) NOT NULL,
    receiver            VARCHAR(9) NOT NULL,

    -- APRS position message
    receiver_time       VARCHAR(7) NOT NULL,
    symbol_table        CHAR NOT NULL,
    symbol_code         CHAR NOT NULL,

    -- parsed APRS position comment
    course              SMALLINT,
    speed               SMALLINT,
    altitude            INTEGER,
    address_type        SMALLINT,
    aircraft_type       SMALLINT,
    is_stealth          BOOLEAN,
    is_notrack          BOOLEAN,
    address             INTEGER,
    climb_rate          INTEGER,
    turn_rate           DOUBLE PRECISION,
    error               SMALLINT,
    frequency_offset    DOUBLE PRECISION,
    signal_quality      DOUBLE PRECISION,
    gps_quality         TEXT,
    flight_level        DOUBLE PRECISION,
    signal_power        DOUBLE PRECISION,
    software_version    DOUBLE PRECISION,
    hardware_version    SMALLINT,
    original_address    INTEGER,

    unparsed            TEXT,

    -- additional (externally calculated) fields
    receiver_ts         TIMESTAMPTZ,
    bearing             DOUBLE PRECISION,
    distance            DOUBLE PRECISION,
    normalized_quality  DOUBLE PRECISION,

    -- additional (externally calculated) field, for PostGIS only
    location            GEOMETRY(POINT, 4326),
    elevation           INTEGER,

    -- bit coded plausibility check
    plausibility        SMALLINT
);
CREATE INDEX idx_positions_src_call ON positions (src_call, ts);

CREATE TABLE IF NOT EXISTS statuses (
    "ts"                TIMESTAMPTZ NOT NULL,

    -- APRS message body
    src_call            VARCHAR(9) NOT NULL,
    dst_call            VARCHAR(9) NOT NULL,
    receiver            VARCHAR(9) NOT NULL,

	-- APRS status message
    receiver_time       VARCHAR(7) NOT NULL,

	-- parsed APRS status comment
    version             TEXT,
    platform            TEXT,
    cpu_load            DOUBLE PRECISION,
    ram_free            DOUBLE PRECISION,
    ram_total           DOUBLE PRECISION,
    ntp_offset	        DOUBLE PRECISION,
    ntp_correction      DOUBLE PRECISION,
    voltage             DOUBLE PRECISION,
    amperage            DOUBLE PRECISION,
    cpu_temperature     DOUBLE PRECISION,
    visible_senders     SMALLINT,
    latency             DOUBLE PRECISION,
    senders             SMALLINT,
    rf_correction_manual        SMALLINT,
    rf_correction_automatic     DOUBLE PRECISION,
    noise                       DOUBLE PRECISION,
    senders_signal_quality      DOUBLE PRECISION,
    senders_messages            INTEGER,
    good_senders_signal_quality DOUBLE PRECISION,
    good_senders                INTEGER,
    good_and_bad_senders        INTEGER,

    unparsed            TEXT,

    -- additional (externally calculated) fields
    receiver_ts         TIMESTAMPTZ
);
CREATE INDEX idx_statuses_src_call ON statuses (src_call, ts);