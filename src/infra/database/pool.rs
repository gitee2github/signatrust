/*
 *
 *  * // Copyright (c) 2023 Huawei Technologies Co.,Ltd. All rights reserved.
 *  * //
 *  * // signatrust is licensed under Mulan PSL v2.
 *  * // You can use this software according to the terms and conditions of the Mulan
 *  * // PSL v2.
 *  * // You may obtain a copy of Mulan PSL v2 at:
 *  * //         http://license.coscl.org.cn/MulanPSL2
 *  * // THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY
 *  * // KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
 *  * // NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *  * // See the Mulan PSL v2 for more details.
 *
 */

use crate::util::error;
use config::Value;
use once_cell::sync::OnceCell;
use sqlx::mysql::{MySql, MySqlPoolOptions};
use sqlx::pool::Pool;
use std::collections::HashMap;

use crate::util::error::{Error, Result};
pub type DbPool = Pool<MySql>;

static DB_POOL: OnceCell<DbPool> = OnceCell::new();

pub async fn create_pool(config: &HashMap<String, Value>) -> Result<()> {
    let max_connections: u32 = config
        .get("max_connection")
        .expect("max connection should configured")
        .to_string()
        .parse()?;
    if max_connections == 0 {
        return Err(Error::ConfigError(format!(
            "max connection for database is incorrect {}",
            max_connections
        )));
    }
    let db_connection = config
        .get("connection_url")
        .expect("database connection url should configured")
        .to_string();
    if db_connection.is_empty() {
        return Err(Error::ConfigError(format!(
            "database connection url is incorrect {}",
            db_connection
        )));
    }
    let pool = MySqlPoolOptions::new()
        .max_connections(max_connections)
        .connect(db_connection.as_str())
        .await
        .map_err(Error::from)?;
    DB_POOL.set(pool).expect("db pool configured");
    ping().await?;
    Ok(())
}

pub fn get_db_pool() -> Result<DbPool> {
    return match DB_POOL.get() {
        None => Err(error::Error::DatabaseError(
            "failed to get database pool".to_string(),
        )),
        Some(pool) => Ok(pool.clone()),
    };
}

pub async fn ping() -> Result<()> {
    info!("Checking on database connection...");
    let pool = get_db_pool();
    match pool {
        Ok(pool) => {
            sqlx::query("SELECT 1")
                .fetch_one(&pool)
                .await
                .expect("Failed to PING database");
            info!("Database PING executed successfully!");
        }
        Err(e) => return Err(Error::DatabaseError(e.to_string())),
    }
    Ok(())
}
