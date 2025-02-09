CREATE TABLE Author (
    Id serial NOT NULL,
    Name varchar(70) NOT NULL,
    Country varchar(100) NOT NULL,
    PRIMARY KEY (Id)
);

INSERT INTO Author (Name, Country)
    VALUES ('Agatha Christie', 'United Kingdom'), ('John Ronald Reuel Tolkien', 'United Kingdom');

CREATE TABLE Book (
    Id serial NOT NULL,
    Title varchar(50) NOT NULL,
    Translations text[] NOT NULL DEFAULT ARRAY['french', 'english'],
    PRIMARY KEY (Id)
);

INSERT INTO Book (Title)
    VALUES ('Murder on the Orient Express'), ('Death on the Nile'), ('The Hobbit'), ('The Silmarillion');

CREATE TABLE BookAuthor (
    AuthorId int NOT NULL,
    BookId int NOT NULL,
    FOREIGN KEY (AuthorId) REFERENCES Author (Id),
    FOREIGN KEY (BookId) REFERENCES Book (Id)
);

INSERT INTO BookAuthor (AuthorId, BookId)
    VALUES (1, 1), (1, 2), (2, 3), (2, 4);

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

INSERT INTO SpongeBobVoiceActor (voice_actor, character)
    VALUES (ROW ('Bill Fagerbakke', 65), 'Patrick');

CREATE TABLE TestType (
    -- 論理型
    col_bool                boolean,
    col_bool_alias          bool,
    col_bool_array          bool[],

    -- 数値型
    col_smallint            smallint,
    col_smallint_alias      int2,
    col_char                "char",

    col_integer             integer,
    col_integer_alias       int4,
    col_int_alias           int,
    col_serial              serial,      -- 自動採番用の擬似型
    col_bigint              bigint,
    col_bigint_alias        int8,

    col_decimal             decimal,
    col_decimal_alias       numeric,

    col_real                real,
    col_real_alias          float4,

    col_double_precision    double precision,
    col_double_precision_alias float8,

    col_money               money,

    -- 文字列・キャラクター系
    col_char                character(10),
    col_char_alias          char(10),

    col_varchar             character varying(50),
    col_varchar_alias       varchar(50),

    col_text                text,

    -- バイナリ型
    col_bytea               bytea,
    col_blob                blob,

    -- UUID 型
    col_uuid                uuid,

    -- 日付・時刻型
    col_date                date,
    col_time                time without time zone,
    col_time_tz             time with time zone,
    col_timestamp           timestamp without time zone,
    col_timestamp_alias     timestamp,   -- 同じ意味
    col_timestamptz         timestamp with time zone,
    col_timestamptz_alias   timestamptz, -- 同じ意味
    col_interval            interval,

    -- JSON 型
    col_json                json,
    col_jsonb               jsonb,

    -- ネットワーク型
    col_inet                inet,
    col_cidr                cidr,
    col_macaddr             macaddr,

    -- 幾何学的型
    col_point               point,
    col_line                line,
    col_lseg                lseg,
    col_box                 box,
    col_path                path,
    col_polygon             polygon,
    col_circle              circle,

    -- ビット文字列型
    col_bit                 bit(8),
    col_varbit              bit varying(8),
    col_varbit_alias        varbit(8),

    -- テキスト検索型
    col_tsvector            tsvector,
    col_tsquery             tsquery,

    -- XML 型
    col_xml                 xml,

    -- Range 型（PostgreSQL 9.2 以降）
    col_int4range           int4range,
    col_numrange            numrange,
    col_daterange           daterange,
    col_tsrange             tsrange,
    col_tstzrange           tstzrange
);