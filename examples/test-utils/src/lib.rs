use test_context::{AsyncTestContext, TestContext};

pub struct PgTokioTestContext {
    db_name: String,
    pub client: tokio_postgres::Client,
}

pub struct PgSyncTestContext {
    db_name: String,
    pub client: postgres::Client,
}

fn get_postgres_url() -> String {
    let host = std::env::var("POSTGRES_HOST").unwrap_or("localhost".to_owned());
    let port = std::env::var("POSTGRES_PORT").unwrap_or("5432".to_owned());
    let db = std::env::var("POSTGRES_DB").unwrap_or("app".to_owned());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or("password".to_owned());
    let user = std::env::var("POSTGRES_USER").unwrap_or("root".to_owned());
    format!("host={host} port={port} user={user} password={password} dbname={db}")
}

fn generate_tmp_db() -> String {
    let db_rand = std::iter::repeat_with(fastrand::alphanumeric)
        .take(10)
        .collect::<String>();
    format!("test_db_{db_rand}").to_lowercase()
}

impl TestContext for PgSyncTestContext {
    fn setup() -> Self {
        let db_url = get_postgres_url();
        dbg!(&db_url);
        let mut admin_client = postgres::Client::connect(&db_url, postgres::NoTls).unwrap();

        let test_db_name = generate_tmp_db();

        admin_client
            .batch_execute(&format!("CREATE DATABASE {test_db_name}"))
            .unwrap();

        let mut config = db_url.parse::<postgres::Config>().unwrap();
        let config = config.dbname(&test_db_name);

        let client = config.connect(postgres::NoTls).unwrap();
        Self {
            client,
            db_name: test_db_name,
        }
    }
    fn teardown(self) {
        drop(self.client);

        let db_url = get_postgres_url();
        let mut admin_client = postgres::Client::connect(&db_url, postgres::NoTls).unwrap();

        admin_client
            .batch_execute(&format!("DROP DATABASE {}", self.db_name))
            .unwrap();
    }
}

impl AsyncTestContext for PgTokioTestContext {
    async fn setup() -> Self {
        let db_url = get_postgres_url();

        let (admin_client, admin_conn) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move {
            if let Err(e) = admin_conn.await {
                panic!("connection error: {}", e);
            }
        });

        let test_db_name = generate_tmp_db();

        let stmt = format!("CREATE DATABASE {test_db_name};");
        admin_client.batch_execute(&stmt).await.unwrap();

        let mut config = db_url.parse::<tokio_postgres::Config>().unwrap();
        let config = config.dbname(test_db_name.clone());

        let (client, conn) = config.connect(tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                panic!("connection error: {}", e);
            }
        });

        Self {
            client,
            db_name: test_db_name,
        }
    }
    async fn teardown(self) {
        drop(self.client);

        let db_url = get_postgres_url();
        let (admin_client, admin_conn) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move {
            if let Err(e) = admin_conn.await {
                panic!("connection error: {}", e);
            }
        });
        let stmt = format!("DROP DATABASE {};", self.db_name);
        admin_client.batch_execute(&stmt).await.unwrap();
    }
}
