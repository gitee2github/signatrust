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

use super::dto::DataKeyDTO;
use crate::infra::database::pool::DbPool;
use crate::domain::datakey::entity::{DataKey, KeyState, Visibility};
use crate::domain::datakey::repository::Repository;
use crate::util::error::{Result};
use async_trait::async_trait;
use std::boxed::Box;
use chrono::{Utc};
use crate::infra::database::model::request_delete::dto::RequestDeleteDTO;

const DELETE_THRESHOLD: i32 = 3;

#[derive(Clone)]
pub struct DataKeyRepository {
    db_pool: DbPool,
}

impl DataKeyRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
        }
    }

    async fn create_request_delete(&self, user_id: i32, id: i32) -> Result<()> {
        let _ : Option<RequestDeleteDTO> = sqlx::query_as("INSERT IGNORE INTO request_delete(user_id, key_id, create_at) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(id)
            .bind(Utc::now())
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn delete_request_delete(&self, user_id: i32, id: i32) -> Result<()> {
        let _ : Option<RequestDeleteDTO> = sqlx::query_as("DELETE FROM request_delete WHERE user_id = ? AND key_id = ?")
            .bind(user_id)
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Repository for DataKeyRepository {
    async fn create(&self, data_key: DataKey) -> Result<DataKey> {
        let dto = DataKeyDTO::try_from(data_key)?;
        let record : u64 = sqlx::query("INSERT INTO data_key(name, description, user, attributes, key_type, fingerprint, private_key, public_key, certificate, create_at, expire_at, key_state, visibility) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&dto.name)
            .bind(&dto.description)
            .bind(dto.user)
            .bind(dto.attributes)
            .bind(dto.key_type)
            .bind(dto.fingerprint)
            .bind(dto.private_key)
            .bind(dto.public_key)
            .bind(dto.certificate)
            .bind(dto.create_at)
            .bind(dto.expire_at)
            .bind(dto.key_state)
            .bind(dto.visibility)
            .execute(&self.db_pool)
            .await?.last_insert_id();
        return self.get_by_id(record as i32).await
    }

    async fn get_public_keys(&self) -> Result<Vec<DataKey>> {
        let dtos: Vec<DataKeyDTO> = sqlx::query_as("SELECT * FROM data_key WHERE key_state != ? and visibility = ?")
            .bind(KeyState::Deleted.to_string())
            .bind(Visibility::Public.to_string())
            .fetch_all(&self.db_pool)
            .await?;
        let mut results = vec![];
        for dto in dtos.into_iter() {
            results.push(DataKey::try_from(dto)?);
        }
        Ok(results)
    }
    async fn get_private_keys(&self, user_id: i32) -> Result<Vec<DataKey>> {
        let dtos: Vec<DataKeyDTO> = sqlx::query_as("SELECT * FROM data_key WHERE key_state != ? and visibility = ? and user = ?")
            .bind(KeyState::Deleted.to_string())
            .bind(Visibility::Private.to_string())
            .bind(user_id)
            .fetch_all(&self.db_pool)
            .await?;
        let mut results = vec![];
        for dto in dtos.into_iter() {
            results.push(DataKey::try_from(dto)?);
        }
        Ok(results)
    }

    async fn get_by_id(&self, id: i32) -> Result<DataKey> {
        let dto: DataKeyDTO = sqlx::query_as("SELECT * FROM data_key WHERE id = ? AND key_state != ?")
            .bind(id)
            .bind(KeyState::Deleted.to_string())
            .fetch_one(&self.db_pool)
            .await?;
        Ok(DataKey::try_from(dto)?)
    }

    async fn update_state(&self, id: i32, state: KeyState) -> Result<()> {
        //Note: if the key in deleted status, it cannot be updated to other states
        let _: Option<DataKeyDTO>  = sqlx::query_as("UPDATE data_key SET key_state = ? WHERE id = ? AND key_state != ?")
            .bind(state.to_string())
            .bind(id)
            .bind(KeyState::Deleted.to_string())
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn get_enabled_key_by_type_and_name(&self, key_type: String, name: String) -> Result<DataKey> {
        let dto: DataKeyDTO = sqlx::query_as("SELECT * FROM data_key WHERE name = ? AND key_type = ? AND key_state = ?")
            .bind(name)
            .bind(key_type)
            .bind(KeyState::Enabled.to_string())
            .fetch_one(&self.db_pool)
            .await?;
        Ok(DataKey::try_from(dto)?)
    }

    async fn delete_private_key(&self, id: i32, user_id: i32) -> Result<()> {
        let _: Option<DataKeyDTO> = sqlx::query_as("UPDATE data_key SET key_state = ? WHERE id = ? AND visibility = ? AND user = ?")
            .bind(KeyState::Deleted.to_string())
            .bind(id)
            .bind(Visibility::Private.to_string())
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn request_delete_public_key(&self, user_id: i32, id: i32) -> Result<()> {
        //use transaction here?
        let _: Option<DataKeyDTO> = sqlx::query_as(
            "UPDATE data_key SET key_state = ? \
            WHERE id = ? AND visibility = ? AND key_state != ?")
            .bind(KeyState::PendingDelete.to_string())
            .bind(id)
            .bind(Visibility::Public.to_string())
            .bind(KeyState::PendingDelete.to_string())
            .fetch_optional(&self.db_pool)
            .await?;
        self.create_request_delete(user_id, id).await?;
        let _: Option<DataKeyDTO> = sqlx::query_as(
            "UPDATE data_key SET key_state = ? \
            WHERE id = ? AND visibility = ? AND ( \
            SELECT COUNT(*) FROM request_delete WHERE key_id = ?) >= ?")
            .bind(KeyState::Deleted.to_string())
            .bind(id)
            .bind(Visibility::Public.to_string())
            .bind(id)
            .bind(DELETE_THRESHOLD)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn cancel_delete_public_key(&self, user_id: i32, id: i32) -> Result<()> {
        self.delete_request_delete(user_id, id).await?;
        let _: Option<DataKeyDTO> = sqlx::query_as(
            "UPDATE data_key SET key_state = ? \
            WHERE id = ? AND visibility = ? AND ( \
            SELECT COUNT(*) FROM request_delete WHERE key_id = ?) = ?")
            .bind(KeyState::Disabled.to_string())
            .bind(id)
            .bind(Visibility::Public.to_string())
            .bind(id)
            .bind(0)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(())
    }
}
