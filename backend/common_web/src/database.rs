use std::error::Error;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection
};

use common::config::Config;

embed_migrations!("./migrations");

pub type DBConnPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_db_conn_pool(config: &Config) -> Result<DBConnPool, Box<dyn Error>> {
    let db_conn_url = format!(
        "{db_type}://{username}:{password}@{host}/{database_name}",
        db_type = config.database_dialect,
        username = config.database_username,
        password = config.database_password,
        host = config.database_host,
        database_name = config.database_name
    );

    let manager = ConnectionManager::new(db_conn_url.as_str());

    let pool = Pool::new(manager)?;

    embedded_migrations::run_with_output(&*pool.get()?, &mut std::io::stdout())?;

    Ok(pool)
}
