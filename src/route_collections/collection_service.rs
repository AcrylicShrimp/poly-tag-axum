use crate::{
    db::DBPool,
    schema::{
        dto_in::{
            CreateCollectionBodyDto, FindCollectionPathDto, FindCollectionsQueryDto,
            PaginationOrderDto, RemoveCollectionPathDto, UpdateCollectionBodyDto,
            UpdateCollectionPathDto,
        },
        dto_out::{CollectionDto, FindCollectionsResultDto, PaginationMetadataDto},
    },
};
use axum::http::StatusCode;
use chrono::NaiveDateTime;
use codegen::ErrorEnum;
use diesel::{
    prelude::*,
    sql_types::{Bool, Integer},
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(ErrorEnum, Error, Debug)]
pub enum CollectionServiceError {
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    DieselError(#[from] diesel::result::Error),
}

#[derive(Clone)]
pub struct CollectionService {
    db_pool: DBPool,
}

impl CollectionService {
    pub fn new(db_pool: DBPool) -> Self {
        Self { db_pool }
    }

    pub async fn find_collections(
        &self,
        query: FindCollectionsQueryDto,
    ) -> Result<FindCollectionsResultDto, CollectionServiceError> {
        use crate::db::schema::collections::dsl::*;

        let db_conn = &mut self.db_pool.get().await?;
        db_conn
            .transaction(|db_conn| {
                async move {
                    let mut q = collections
                        .select((id, uuid, name, description, created_at))
                        .limit(query.page_size as i64)
                        .into_boxed();

                    if let Some(first_id) = query.first_id {
                        match query.order {
                            PaginationOrderDto::Asc => {
                                q = q.filter(id.lt(first_id));
                            }
                            PaginationOrderDto::Desc => {
                                q = q.filter(id.gt(first_id));
                            }
                        }
                    }

                    if let Some(last_id) = query.last_id {
                        match query.order {
                            PaginationOrderDto::Asc => {
                                q = q.filter(id.gt(last_id));
                            }
                            PaginationOrderDto::Desc => {
                                q = q.filter(id.lt(last_id));
                            }
                        }
                    }

                    match query.order {
                        PaginationOrderDto::Asc => {
                            q = q.order(id.asc());
                        }
                        PaginationOrderDto::Desc => {
                            q = q.order(id.desc());
                        }
                    }

                    if let Some(filter_name) = &query.filter_name {
                        q = q.filter(name.eq(filter_name));
                    }

                    let raw_items = q.load::<RawCollectionDto>(db_conn).await?;
                    let has_prev_q = match query.order {
                        PaginationOrderDto::Asc => {
                            let mut q = collections
                                .select(diesel::dsl::sql::<Integer>("1"))
                                .into_boxed();

                            if let Some(first) = raw_items.as_slice().first() {
                                q = q.filter(id.lt(first.id));
                            } else {
                                q = q.filter(diesel::dsl::sql::<Bool>("false"));
                            }

                            if let Some(filter_name) = &query.filter_name {
                                q = q.filter(name.eq(filter_name));
                            }

                            q
                        }
                        PaginationOrderDto::Desc => {
                            let mut q = collections
                                .select(diesel::dsl::sql::<Integer>("1"))
                                .into_boxed();

                            if let Some(last) = raw_items.as_slice().last() {
                                q = q.filter(id.gt(last.id));
                            } else {
                                q = q.filter(diesel::dsl::sql::<Bool>("false"));
                            }

                            if let Some(filter_name) = &query.filter_name {
                                q = q.filter(name.eq(filter_name));
                            }

                            q
                        }
                    };
                    let has_next_q = match query.order {
                        PaginationOrderDto::Asc => {
                            let mut q = collections
                                .select(diesel::dsl::sql::<Integer>("1"))
                                .into_boxed();

                            if let Some(last) = raw_items.as_slice().last() {
                                q = q.filter(id.gt(last.id));
                            } else {
                                q = q.filter(diesel::dsl::sql::<Bool>("false"));
                            }

                            if let Some(filter_name) = &query.filter_name {
                                q = q.filter(name.eq(filter_name));
                            }

                            q
                        }
                        PaginationOrderDto::Desc => {
                            let mut q = collections
                                .select(diesel::dsl::sql::<Integer>("1"))
                                .into_boxed();

                            if let Some(first) = raw_items.as_slice().first() {
                                q = q.filter(id.lt(first.id));
                            } else {
                                q = q.filter(diesel::dsl::sql::<Bool>("false"));
                            }

                            if let Some(filter_name) = &query.filter_name {
                                q = q.filter(name.eq(filter_name));
                            }

                            q
                        }
                    };

                    let has_prev_q = diesel::select(diesel::dsl::exists(has_prev_q));
                    let has_next_q = diesel::select(diesel::dsl::exists(has_next_q));

                    let has_prev = has_prev_q.get_result::<bool>(db_conn).await?;
                    let has_next = has_next_q.get_result::<bool>(db_conn).await?;
                    let pagination = PaginationMetadataDto { has_prev, has_next };

                    let items = raw_items
                        .into_iter()
                        .map(|raw_item| raw_item.into())
                        .collect();

                    Ok(FindCollectionsResultDto { pagination, items })
                }
                .scope_boxed()
            })
            .await
    }

    pub async fn find_collection(
        &self,
        path: FindCollectionPathDto,
    ) -> Result<Option<CollectionDto>, CollectionServiceError> {
        use crate::db::schema::collections::dsl::*;

        let db_conn = &mut self.db_pool.get().await?;
        let raw_item = collections
            .filter(uuid.eq(path.identifier))
            .get_result::<RawCollectionDto>(db_conn)
            .await
            .optional()?;

        Ok(raw_item.map(|item| item.into()))
    }

    pub async fn create_collection(
        &self,
        body: CreateCollectionBodyDto,
    ) -> Result<CollectionDto, CollectionServiceError> {
        use crate::db::schema::collections::dsl::*;

        let db_conn = &mut self.db_pool.get().await?;
        let raw_item = diesel::insert_into(collections)
            .values((name.eq(body.name), description.eq(body.description)))
            .get_result::<RawCollectionDto>(db_conn)
            .await?;

        Ok(raw_item.into())
    }

    pub async fn update_collection(
        &self,
        path: UpdateCollectionPathDto,
        body: UpdateCollectionBodyDto,
    ) -> Result<Option<CollectionDto>, CollectionServiceError> {
        use crate::db::schema::collections::dsl::*;

        let db_conn = &mut self.db_pool.get().await?;
        let raw_item = diesel::update(collections.filter(id.eq(path.identifier)))
            .set((name.eq(body.name), description.eq(body.description)))
            .get_result::<RawCollectionDto>(db_conn)
            .await
            .optional()?;

        Ok(raw_item.map(|item| item.into()))
    }

    pub async fn remove_collection(
        &self,
        path: RemoveCollectionPathDto,
    ) -> Result<Option<CollectionDto>, CollectionServiceError> {
        use crate::db::schema::collections::dsl::*;

        let db_conn = &mut self.db_pool.get().await?;
        let raw_item = diesel::delete(collections.filter(id.eq(path.identifier)))
            .get_result::<RawCollectionDto>(db_conn)
            .await
            .optional()?;

        Ok(raw_item.map(|item| item.into()))
    }
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RawCollectionDto {
    id: i32,
    uuid: Uuid,
    name: String,
    description: Option<String>,
    created_at: NaiveDateTime,
}

impl From<CollectionDto> for RawCollectionDto {
    fn from(item: CollectionDto) -> Self {
        Self {
            id: item.id,
            uuid: item.uuid,
            name: item.name,
            description: item.description,
            created_at: item.created_at.naive_utc(),
        }
    }
}

impl From<RawCollectionDto> for CollectionDto {
    fn from(item: RawCollectionDto) -> Self {
        Self {
            id: item.id,
            uuid: item.uuid,
            name: item.name,
            description: item.description,
            created_at: item.created_at.and_utc(),
        }
    }
}
