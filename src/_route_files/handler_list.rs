use crate::{
    app_state::AppState,
    db::{
        model::{FileUuid, FileWithTags, TagTemplateCompact, TagValueTypeKind},
        DBPool,
    },
};
use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg,
    prelude::*,
    query_builder::{BoxedSqlQuery, SqlQuery},
    sql_query,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use dto::*;
use meilisearch_sdk::{
    search::{SearchQuery, Selectors},
    Client,
};
use std::sync::Arc;
use uuid::Uuid;

/// List files with optional filters.
#[utoipa::path(
    get,
    operation_id = "file-list",
    tag = "file",
    path = "/files",
    request_body = FileListReqBody,
    responses(
        (status = OK, description = "Ok", body = FileListRes),
        (status = UNPROCESSABLE_ENTITY, description = "Invalid request has been received", body = ErrorBody, example = json!({
            "error": "tag template `550e8400-e29b-41d4-a716-446655440000` is duplicated"
        })),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody),
    ),
)]
#[debug_handler(state = AppState)]
pub async fn handle(
    State(db_pool): State<DBPool>,
    State(meilisearch_client): State<Arc<Client>>,
    Query(query): Query<QueryParam>,
    Json(mut body): Json<FileListReqBody>,
) -> Result<(StatusCode, Json<FileListRes>), ErrRes> {
    use crate::db::schema::files::dsl as files;
    use crate::db::schema::tag_templates::dsl as tag_templates;

    const PAGE_SIZE: u32 = 40;
    const FILE_UUID_TABLE_NAME: &str = "file_uuids";

    let uuids = if let Some(query) = &body.query {
        let query = query.trim();

        if query.is_empty() {
            None
        } else {
            Some(
                SearchQuery::execute::<FileUuid>(
                    meilisearch_client
                        .index("files")
                        .search()
                        .with_attributes_to_retrieve(Selectors::Some(&["uuid"]))
                        .with_attributes_to_highlight(Selectors::Some(&[]))
                        .with_limit(200)
                        .with_query(query),
                )
                .await?
                .hits,
            )
        }
    } else {
        None
    };

    let db_connection = &mut db_pool.get().await?;
    db_connection
        .transaction(|db_connection| {
            async move {
                let uuid_table_name = if let Some(tags) = &mut body.tags {
                    if !tags.is_empty() {
                        tags.sort_by_key(|tag| tag.template_uuid);

                        for index in 1..tags.len() {
                            if tags[index - 1].template_uuid == tags[index].template_uuid {
                                return Err(ErrRes::DuplicatedTagTemplate(
                                    tags[index].template_uuid,
                                ));
                            }
                        }

                        let templates = tag_templates::tag_templates
                            .select((tag_templates::uuid, tag_templates::value_type))
                            .filter(
                                tag_templates::uuid
                                    .eq_any(tags.iter().map(|tag| tag.template_uuid)),
                            )
                            .order_by(tag_templates::uuid.asc())
                            .load::<TagTemplateCompact>(db_connection)
                            .await?;

                        if templates.len() != tags.len() {
                            for index in 0..tags.len() {
                                if templates.len() <= index {
                                    return Err(ErrRes::InvalidTagTemplate(
                                        tags[index].template_uuid,
                                    ));
                                }

                                if tags[index].template_uuid != templates[index].uuid {
                                    return Err(ErrRes::InvalidTagTemplate(
                                        tags[index].template_uuid,
                                    ));
                                }
                            }
                        }

                        for index in 0..templates.len() {
                            match (templates[index].value_type, &tags[index].value) {
                                (None, Some(_)) => {
                                    return Err(ErrRes::ExtraTagValueFilter(templates[index].uuid));
                                }
                                (Some(value_ty), Some(value)) => {
                                    value.check_value_type(value_ty).map_err(|err_value_ty| {
                                        ErrRes::InvalidTagValueFilter(
                                            templates[index].uuid,
                                            value_ty,
                                            err_value_ty,
                                        )
                                    })?;
                                }
                                _ => {}
                            }
                        }

                        // Create a temporary table to store file UUIDs.
                        create_file_uuid_table_sql(FILE_UUID_TABLE_NAME)
                            .execute(db_connection)
                            .await?;

                        // Fill the temporary table with file UUIDs which have all the tags.
                        insert_file_uuid_table_sql(FILE_UUID_TABLE_NAME, &templates, tags)
                            .execute(db_connection)
                            .await?;

                        // Create a unique index on the temporary table.
                        create_index_file_uuid_table_sql(FILE_UUID_TABLE_NAME)
                            .execute(db_connection)
                            .await?;

                        // Run an analysis on the temporary table.
                        analyze_file_uuid_table_sql(FILE_UUID_TABLE_NAME)
                            .execute(db_connection)
                            .await?;

                        FILE_UUID_TABLE_NAME
                    } else {
                        "files"
                    }
                } else {
                    "files"
                };

                let page = query.page.unwrap_or_default();
                let uuids = select_file_uuid_from_table_sql(
                    uuid_table_name,
                    page,
                    PAGE_SIZE,
                    uuids
                        .as_ref()
                        .map(|uuids| uuids.iter().map(|hit| hit.result.uuid)),
                )
                .load::<FileUuid>(db_connection)
                .await?;

                let files = if uuids.is_empty() {
                    vec![]
                } else {
                    files::files
                        .select((
                            files::uuid,
                            files::name,
                            files::mime.assume_not_null(),
                            files::size.assume_not_null(),
                            files::hash.assume_not_null(),
                            files::uploaded_at,
                        ))
                        .filter(
                            files::mime
                                .is_not_null()
                                .and(files::size.is_not_null())
                                .and(files::hash.is_not_null()),
                        )
                        .order_by(files::uuid.asc())
                        .load::<FileWithTags>(db_connection)
                        .await?
                        .into_iter()
                        .map(|file| FileListResFile {
                            uuid: file.uuid,
                            name: file.name,
                            mime: file.mime,
                            size: file.size as u64,
                            hash: file.hash,
                            // NOTE: We're using diesel, and diesel uses UTC by default.
                            // See: https://github.com/diesel-rs/diesel/issues/1024
                            uploaded_at: DateTime::<Utc>::from_naive_utc_and_offset(
                                file.uploaded_at,
                                Utc,
                            ),
                        })
                        .collect()
                };

                Ok((StatusCode::OK, Json(FileListRes { page, items: files })))
            }
            .scope_boxed()
        })
        .await
}

fn create_file_uuid_table_sql(table_name: impl AsRef<str>) -> SqlQuery {
    sql_query(format!(
        "CREATE TEMP TABLE {} ( uuid UUID NOT NULL ) ON COMMIT DROP",
        table_name.as_ref()
    ))
}

fn insert_file_uuid_table_sql<'a>(
    table_name: impl AsRef<str>,
    templates: &[TagTemplateCompact],
    tags: &'a [FileListReqBodyTag],
) -> BoxedSqlQuery<'a, Pg, SqlQuery> {
    debug_assert!(!tags.is_empty());
    debug_assert!(tags.len() == templates.len());

    let mut query = sql_query(format!(
        "INSERT INTO {} SELECT file_uuid FROM tags WHERE FALSE",
        table_name.as_ref(),
    ))
    .into_boxed();

    for (template, tag) in templates.iter().zip(tags.iter()) {
        query = query
            .sql("OR (template_uuid = ?")
            .bind::<diesel::sql_types::Uuid, _>(tag.template_uuid);

        match (template.value_type, &tag.value) {
            (Some(value_type), Some(value)) => {
                query = filter_tag_value_sql(query, value_type, value);
            }
            _ => {}
        }

        query = query.sql(")");
    }

    query
        .sql("GROUP BY file_uuid HAVING COUNT(file_uuid) = ?")
        .bind::<diesel::sql_types::BigInt, _>(tags.len() as i64)
}

fn filter_tag_value_sql<'a>(
    mut query: BoxedSqlQuery<'a, Pg, SqlQuery>,
    value_ty: TagValueTypeKind,
    value: &'a FileListReqBodyTagValue,
) -> BoxedSqlQuery<'a, Pg, SqlQuery> {
    let column_name = value_ty.into_column_name();

    if let Some(equal) = &value.equal {
        query = equal.attach_value(query.sql(format!("AND ({} = ?)", column_name)));
    }

    if let Some(not_equal) = &value.not_equal {
        query = not_equal.attach_value(query.sql(format!("AND ({} != ?)", column_name)));
    }

    if let Some(less_than) = &value.less_than {
        query = less_than.attach_value(query.sql(format!("AND ({} < ?)", column_name)));
    }

    if let Some(less_than_or_equal) = &value.less_than_or_equal {
        query = less_than_or_equal.attach_value(query.sql(format!("AND ({} <= ?)", column_name)));
    }

    if let Some(greater_than) = &value.greater_than {
        query = greater_than.attach_value(query.sql(format!("AND ({} > ?)", column_name)));
    }

    if let Some(greater_than_or_equal) = &value.greater_than_or_equal {
        query =
            greater_than_or_equal.attach_value(query.sql(format!("AND ({} >= ?)", column_name)));
    }

    if let Some(contains) = &value.contains {
        query = contains.attach_value(query.sql(format!("AND ({} LIKE ?)", column_name)));
    }

    if let Some(one_of) = &value.one_of {
        if !one_of.is_empty() {
            query = query.sql("AND ({} IN (");

            for index in 0..one_of.len() {
                if index != 0 {
                    query = query.sql(", ");
                }

                query = one_of[index].attach_value(query.sql("?"));
            }

            query = query.sql("))");
        }
    }

    query
}

fn create_index_file_uuid_table_sql(table_name: impl AsRef<str>) -> SqlQuery {
    sql_query(format!(
        "CREATE UNIQUE INDEX ON {} ( uuid ASC )",
        table_name.as_ref()
    ))
}

fn analyze_file_uuid_table_sql(table_name: impl AsRef<str>) -> SqlQuery {
    sql_query(format!("ANALYZE {}", table_name.as_ref()))
}

fn select_file_uuid_from_table_sql(
    table_name: impl AsRef<str>,
    page: u32,
    page_size: u32,
    uuids: Option<impl Iterator<Item = Uuid>>,
) -> SqlQuery {
    sql_query(format!(
        "SELECT uuid FROM {} {} ORDER BY uuid ASC OFFSET {} LIMIT {}",
        table_name.as_ref(),
        if let Some(uuids) = uuids {
            // It is safe not to escape the UUIDs because they are UUIDs; they cannot be used for SQL injection.
            let uuids = uuids.map(|uuid| format!("'{}'", uuid)).collect::<Vec<_>>();

            if uuids.is_empty() {
                format!("WHERE FALSE")
            } else {
                format!("WHERE uuid IN ({})", uuids.join(", "))
            }
        } else {
            "".to_owned()
        },
        page * page_size,
        page_size,
    ))
}

pub mod dto {
    use crate::db::model::TagValueTypeKind;
    use axum::http::StatusCode;
    use chrono::{DateTime, Utc};
    use codegen::ErrorEnum;
    use diesel::{
        pg::Pg,
        query_builder::{BoxedSqlQuery, SqlQuery},
    };
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use utoipa::{IntoParams, ToSchema};
    use uuid::Uuid;

    #[derive(Deserialize, IntoParams)]
    #[serde(rename_all = "camelCase")]
    #[into_params(parameter_in = Query)]
    pub struct QueryParam {
        pub page: Option<u32>,
    }

    #[derive(Deserialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FileListReqBody {
        #[schema(example = "john wick")]
        pub query: Option<String>,
        pub tags: Option<Vec<FileListReqBodyTag>>,
    }

    #[derive(Deserialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FileListReqBodyTag {
        pub template_uuid: Uuid,
        pub value: Option<FileListReqBodyTagValue>,
    }

    #[derive(Deserialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FileListReqBodyTagValue {
        pub equal: Option<FileListReqBodyTagValueParam>,
        pub not_equal: Option<FileListReqBodyTagValueParam>,
        pub less_than: Option<FileListReqBodyTagValueParam>,
        pub less_than_or_equal: Option<FileListReqBodyTagValueParam>,
        pub greater_than: Option<FileListReqBodyTagValueParam>,
        pub greater_than_or_equal: Option<FileListReqBodyTagValueParam>,
        pub contains: Option<FileListReqBodyTagValueParam>,
        pub one_of: Option<Vec<FileListReqBodyTagValueParam>>,
    }

    impl FileListReqBodyTagValue {
        pub fn check_value_type(&self, value_ty: TagValueTypeKind) -> Result<(), TagValueTypeKind> {
            if let Some(equal) = &self.equal {
                equal.check_value_type(value_ty)?;
            }

            if let Some(not_equal) = &self.not_equal {
                not_equal.check_value_type(value_ty)?;
            }

            if let Some(less_than) = &self.less_than {
                less_than.check_value_type(value_ty)?;
            }

            if let Some(less_than_or_equal) = &self.less_than_or_equal {
                less_than_or_equal.check_value_type(value_ty)?;
            }

            if let Some(greater_than) = &self.greater_than {
                greater_than.check_value_type(value_ty)?;
            }

            if let Some(greater_than_or_equal) = &self.greater_than_or_equal {
                greater_than_or_equal.check_value_type(value_ty)?;
            }

            if let Some(contains) = &self.contains {
                contains.check_value_type(value_ty)?;
            }

            if let Some(one_of) = &self.one_of {
                for elem in one_of {
                    elem.check_value_type(value_ty)?;
                }
            }

            Ok(())
        }
    }

    #[derive(Deserialize, ToSchema)]
    #[serde(untagged)]
    pub enum FileListReqBodyTagValueParam {
        String(String),
        Integer(i64),
        Boolean(bool),
    }

    impl FileListReqBodyTagValueParam {
        pub fn type_kind(&self) -> TagValueTypeKind {
            match self {
                Self::String(_) => TagValueTypeKind::String,
                Self::Integer(_) => TagValueTypeKind::Integer,
                Self::Boolean(_) => TagValueTypeKind::Boolean,
            }
        }

        pub fn check_value_type(&self, value_ty: TagValueTypeKind) -> Result<(), TagValueTypeKind> {
            let self_type_kind = self.type_kind();

            if self_type_kind != value_ty {
                return Err(self_type_kind);
            }

            Ok(())
        }

        pub fn attach_value<'a>(
            &'a self,
            query: BoxedSqlQuery<'a, Pg, SqlQuery>,
        ) -> BoxedSqlQuery<'a, Pg, SqlQuery> {
            match self {
                FileListReqBodyTagValueParam::String(value) => {
                    query.bind::<diesel::sql_types::Text, _>(value)
                }
                FileListReqBodyTagValueParam::Integer(value) => {
                    query.bind::<diesel::sql_types::BigInt, _>(*value)
                }
                FileListReqBodyTagValueParam::Boolean(value) => {
                    query.bind::<diesel::sql_types::Bool, _>(*value)
                }
            }
        }
    }

    #[derive(Serialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FileListRes {
        #[schema(example = "0")]
        pub page: u32,
        pub items: Vec<FileListResFile>,
    }

    #[derive(Serialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FileListResFile {
        #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
        pub uuid: Uuid,
        #[schema(example = "file.txt")]
        pub name: String,
        #[schema(example = "text/plain")]
        pub mime: String,
        #[schema(example = "1024")]
        pub size: u64,
        #[schema(example = "1234567890")]
        pub hash: i64,
        pub uploaded_at: DateTime<Utc>,
    }

    #[derive(ErrorEnum, Error, Debug)]
    pub enum ErrRes {
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        DieselError(#[from] diesel::result::Error),
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        MeilisearchError(#[from] meilisearch_sdk::errors::Error),
        #[error("tag template `{0}` is duplicated")]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        DuplicatedTagTemplate(Uuid),
        #[error("tag template `{0}` does not exist")]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        InvalidTagTemplate(Uuid),
        #[error("tag template `{0}` does not accept any values, but a value filter was supplied")]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        ExtraTagValueFilter(Uuid),
        #[error("tag template `{0}` expects a value of type `{1}`, but a value filter of type `{2}` was supplied")]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        InvalidTagValueFilter(Uuid, TagValueTypeKind, TagValueTypeKind),
    }
}
