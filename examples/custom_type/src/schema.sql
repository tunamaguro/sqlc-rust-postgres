CREATE TYPE Sponge_Bob_Character AS enum (
    'Bob',
    'Patrick',
    'Squidward'
);

CREATE TYPE VoiceActor AS (
    name text,
    age integer
);

CREATE TABLE SpongeBobVoiceActor (
    voice_actor VoiceActor,
    character Sponge_Bob_Character
);

CREATE TABLE BoolTable(
    col_bool                boolean,
    col_bool_alias          bool,
    col_bool_array1         bool[],
    col_bool_array2         bool[][]
);

CREATE TABLE NumericTable(
    col_smallint            smallint,
    col_smallint_alias      int2,

    col_integer             integer,
    col_integer_alias       int4,
    col_int_alias           int,
    col_serial              serial,   
    col_bigint              bigint,
    col_bigint_alias        int8,

    col_decimal             decimal,
    col_decimal_alias       numeric,

    col_real                real,
    col_real_alias          float4,

    col_double_precision    double precision,
    col_double_precision_alias float8,

    col_money               money
);

CREATE TABLE CharacterTable(
    col_char                character(10),
    col_char_alias          char(10),

    col_varchar             character varying(50),
    col_varchar_alias       varchar(50),

    col_text                text
);

CREATE TABLE BinaryTable(
    col_bytea               bytea
);