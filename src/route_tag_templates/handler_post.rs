use crate::{app_state::AppState, db::DBPool};
use axum::{debug_handler, extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use dto::*;
use uuid::Uuid;

/// Create a new tag template.
#[utoipa::path(
    post,
    operation_id = "tag-template-post",
    tag = "tag-template",
    path = "/tag-templates",
    request_body = TagTemplatePostReqBody,
    responses(
        (status = CREATED, description = "A new tag template has been created", body = TagTemplatePostRes),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody, example = json!({ "error": "internal server error" })),
    ),
)]
#[debug_handler(state = AppState)]
pub async fn handle(
    State(db_pool): State<DBPool>,
    Json(body): Json<TagTemplatePostReqBody>,
) -> Result<(StatusCode, Json<TagTemplatePostRes>), ErrRes> {
    use crate::db::schema::tag_templates::dsl::*;

    let db_connection = &mut db_pool.get().await?;
    let tag_template_uuid = Uuid::new_v4();
    diesel::insert_into(tag_templates)
        .values((
            uuid.eq(tag_template_uuid),
            name.eq(body.name),
            description.eq(body.description),
            value_type.eq(body.value_type),
        ))
        .execute(db_connection)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(TagTemplatePostRes {
            uuid: tag_template_uuid,
        }),
    ))
}

pub mod dto {
    use axum::http::StatusCode;
    use codegen::ErrorEnum;
    use diesel_derive_enum::DbEnum;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use utoipa::ToSchema;
    use uuid::Uuid;

    #[derive(Deserialize, ToSchema)]
    #[schema(example = json!({
        "name": "Author",
        "description": "author of the file",
        "value_type": "string",
    }))]
    pub struct TagTemplatePostReqBody {
        pub name: String,
        pub description: Option<String>,
        pub value_type: Option<TagTemplatePostReqBodyValueTypeKind>,
    }

    #[derive(DbEnum, Deserialize, ToSchema, Debug)]
    #[ExistingTypePath = "crate::db::schema::sql_types::TagValueType"]
    pub enum TagTemplatePostReqBodyValueTypeKind {
        #[serde(alias = "string")]
        String,
        #[serde(alias = "int")]
        #[serde(alias = "integer")]
        Integer,
        #[serde(alias = "bool")]
        #[serde(alias = "boolean")]
        Boolean,
    }

    #[derive(Serialize, ToSchema)]
    pub struct TagTemplatePostRes {
        pub uuid: Uuid,
    }

    #[derive(ErrorEnum, Error, Debug)]
    pub enum ErrRes {
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        DieselError(#[from] diesel::result::Error),
    }
}
