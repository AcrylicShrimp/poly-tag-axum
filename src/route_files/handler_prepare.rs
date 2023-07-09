use crate::{
    app_state::AppState,
    db::{model::TagTemplateCompact, DBPool},
};
use axum::{debug_handler, extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use dto::*;
use uuid::Uuid;

/// Prepare a file to be uploaded.
#[utoipa::path(
    post,
    operation_id = "file-prepare",
    tag = "file",
    path = "/files",
    request_body = FilePrepareReqBody,
    responses(
        (status = CREATED, description = "A new file has been created", body = FilePrepareRes),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody),
    ),
)]
#[debug_handler(state = AppState)]
pub async fn handle(
    State(db_pool): State<DBPool>,
    Json(mut body): Json<FilePrepareReqBody>,
) -> Result<(StatusCode, Json<FilePrepareRes>), ErrRes> {
    use crate::db::schema::files::dsl as files;
    use crate::db::schema::tag_templates::dsl as tag_templates;
    use crate::db::schema::tags::dsl as tags;

    if body.name.len() == 0 {
        return Err(ErrRes::FilenameTooShort(body.name));
    }

    body.tags.sort_by_key(|tag| tag.template_uuid);

    for index in 1..body.tags.len() {
        if body.tags[index - 1].template_uuid == body.tags[index].template_uuid {
            return Err(ErrRes::DuplicatedTagTemplate(
                body.tags[index].template_uuid,
            ));
        }
    }

    let db_connection = &mut db_pool.get().await?;
    let templates = tag_templates::tag_templates
        .select((tag_templates::uuid, tag_templates::value_type))
        .filter(tag_templates::uuid.eq_any(body.tags.iter().map(|tag| tag.template_uuid)))
        .order_by(tag_templates::uuid.asc())
        .load::<TagTemplateCompact>(db_connection)
        .await?;

    for index in 0..templates.len() {
        if templates[index].uuid != body.tags[index].template_uuid {
            return Err(ErrRes::InvalidTagTemplate(body.tags[index].template_uuid));
        }

        match (templates[index].value_type, body.tags[index].value.as_ref()) {
            (Some(template_type_kind), Some(tag_value)) => {
                let type_kind = tag_value.type_kind();
                if template_type_kind != type_kind {
                    return Err(ErrRes::InvalidTagValue(
                        templates[index].uuid,
                        template_type_kind,
                        type_kind,
                    ));
                }
            }
            (Some(template_type_kind), None) => {
                return Err(ErrRes::MissingTagValue(
                    templates[index].uuid,
                    template_type_kind,
                ))
            }
            (None, Some(tag_value)) => {
                return Err(ErrRes::ExtraTagValue(
                    templates[index].uuid,
                    tag_value.type_kind(),
                ))
            }
            (None, None) => {}
        }
    }

    let file_uuid = Uuid::new_v4();

    db_connection
        .transaction(|db_connection| {
            async move {
                diesel::insert_into(files::files)
                    .values((
                        files::uuid.eq(file_uuid),
                        files::name.eq(body.name.as_str()),
                    ))
                    .execute(db_connection)
                    .await?;

                let query = diesel::insert_into(tags::tags);
                let mut filled_query = None;

                for tag in &body.tags {
                    let (value_string, value_integer, value_boolean) = match &tag.value {
                        Some(tag_value) => match tag_value {
                            FilePrepareReqBodyTagValue::String(value) => {
                                (Some(tags::value_string.eq(value)), None, None)
                            }
                            FilePrepareReqBodyTagValue::Integer(value) => {
                                (None, Some(tags::value_integer.eq(value)), None)
                            }
                            FilePrepareReqBodyTagValue::Boolean(value) => {
                                (None, None, Some(tags::value_boolean.eq(value)))
                            }
                        },
                        None => (None, None, None),
                    };
                    filled_query = Some(query.values((
                        tags::template_uuid.eq(tag.template_uuid),
                        tags::file_uuid.eq(file_uuid),
                        value_string,
                        value_integer,
                        value_boolean,
                    )));
                }

                if let Some(filled_query) = filled_query {
                    filled_query.execute(db_connection).await?;
                }

                Ok((StatusCode::OK, Json(FilePrepareRes { uuid: file_uuid })))
            }
            .scope_boxed()
        })
        .await
}

pub mod dto {
    use crate::db::model::TagValueTypeKind;
    use axum::http::StatusCode;
    use codegen::ErrorEnum;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use utoipa::ToSchema;
    use uuid::Uuid;

    #[derive(Deserialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FilePrepareReqBody {
        #[schema(example = "Foo.txt")]
        pub name: String,
        pub tags: Vec<FilePrepareReqBodyTag>,
    }

    #[derive(Deserialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FilePrepareReqBodyTag {
        pub template_uuid: Uuid,
        pub value: Option<FilePrepareReqBodyTagValue>,
    }

    #[derive(Deserialize, ToSchema)]
    #[serde(untagged)]
    pub enum FilePrepareReqBodyTagValue {
        String(String),
        Integer(i64),
        Boolean(bool),
    }

    impl FilePrepareReqBodyTagValue {
        pub fn type_kind(&self) -> TagValueTypeKind {
            match self {
                FilePrepareReqBodyTagValue::String(_) => TagValueTypeKind::String,
                FilePrepareReqBodyTagValue::Integer(_) => TagValueTypeKind::Integer,
                FilePrepareReqBodyTagValue::Boolean(_) => TagValueTypeKind::Boolean,
            }
        }
    }

    #[derive(Serialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FilePrepareRes {
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
        #[error("filename `{0}` is too short")]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        FilenameTooShort(String),
        #[error("tag template `{0}` is duplicated")]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        DuplicatedTagTemplate(Uuid),
        #[error("tag template `{0}` does not exist")]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        InvalidTagTemplate(Uuid),
        #[error(
            "tag template `{0}` does not accept any values, but a value of type `{1}` was supplied"
        )]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        ExtraTagValue(Uuid, TagValueTypeKind),
        #[error("tag template `{0}` requires a value of type `{1}` but was not met")]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        MissingTagValue(Uuid, TagValueTypeKind),
        #[error("tag template `{0}` expects a value of type `{1}`, but a value of type `{2}` was supplied")]
        #[status(StatusCode::UNPROCESSABLE_ENTITY)]
        InvalidTagValue(Uuid, TagValueTypeKind, TagValueTypeKind),
    }
}
