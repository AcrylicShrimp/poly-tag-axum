use crate::{
    app_state::AppState,
    db::{model::TagTemplate, DBPool},
};
use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use dto::*;

/// List tag templates.
#[utoipa::path(
    get,
    operation_id = "tag-template-list",
    tag = "tag-template",
    path = "/tag-templates",
    params(
        QueryParam,
    ),
    responses(
        (status = OK, description = "Tag templates are successfully fetched", body = TagTemplateListRes),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody),
    ),
)]
#[debug_handler(state = AppState)]
pub async fn handle(
    State(db_pool): State<DBPool>,
    Query(query): Query<QueryParam>,
) -> Result<(StatusCode, Json<TagTemplateListRes>), ErrRes> {
    use crate::db::schema::tag_templates::dsl::*;

    const PAGE_SIZE: u32 = 40;

    let db_connection = &mut db_pool.get().await?;
    let templates = tag_templates
        .select((uuid, name, description, value_type, created_at))
        .order((name.asc(), created_at.desc(), uuid.asc()))
        .offset(query.page.unwrap_or(0) as i64 * 40)
        .limit(PAGE_SIZE as i64)
        .load::<TagTemplate>(db_connection)
        .await?;

    Ok((
        StatusCode::OK,
        Json(TagTemplateListRes {
            page: query.page.unwrap_or(0),
            items: templates,
        }),
    ))
}

pub mod dto {
    use crate::db::model::TagTemplate;
    use axum::http::StatusCode;
    use codegen::ErrorEnum;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use utoipa::{IntoParams, ToSchema};

    #[derive(Deserialize, IntoParams)]
    #[serde(rename_all = "camelCase")]
    #[into_params(parameter_in = Query)]
    pub struct QueryParam {
        pub page: Option<u32>,
    }

    #[derive(Serialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct TagTemplateListRes {
        #[schema(example = "0")]
        pub page: u32,
        pub items: Vec<TagTemplate>,
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
