use crate::app_state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub mod collection_service;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/collections", get(handlers::find_collections))
        .route("/collections/:identifier", get(handlers::find_collection))
        .route("/collections", post(handlers::create_collection))
        .route("/collections/:identifier", put(handlers::update_collection))
        .route(
            "/collections/:identifier",
            delete(handlers::remove_collection),
        )
}

pub mod handlers {
    use super::collection_service::{CollectionService, CollectionServiceError};
    use crate::{
        app_state::AppState,
        schema::{
            dto_in::{
                CreateCollectionBodyDto, FindCollectionPathDto, FindCollectionsQueryDto,
                RemoveCollectionPathDto, UpdateCollectionBodyDto, UpdateCollectionPathDto,
            },
            dto_out::{CollectionDto, FindCollectionsResultDto},
        },
    };
    use axum::{
        debug_handler,
        extract::{Path, Query, State},
        http::StatusCode,
        response::{IntoResponse, Response},
        Json,
    };

    /// Finds collections.
    #[utoipa::path(
        get,
        operation_id = "find-collections",
        tag = "collection",
        path = "/collections",
        params(
            FindCollectionsQueryDto
        ),
        responses(
            (status = OK, body = FindCollectionsResultDto),
            (status = INTERNAL_SERVER_ERROR, description = "an error has occurred", body = ErrorBody),
        ),
    )]
    #[debug_handler(state = AppState)]
    pub async fn find_collections(
        State(collection_service): State<CollectionService>,
        Query(query): Query<FindCollectionsQueryDto>,
    ) -> Result<(StatusCode, Json<FindCollectionsResultDto>), CollectionServiceError> {
        let result = collection_service.find_collections(query).await?;

        Ok((StatusCode::OK, Json(result)))
    }

    /// Find a collection.
    #[utoipa::path(
        get,
        operation_id = "find-collection",
        tag = "collection",
        path = "/collections/{identifier}",
        params(
            FindCollectionPathDto
        ),
        responses(
            (status = OK, body = CollectionDto),
            (status = NOT_FOUND, description = "the collection does not exist"),
            (status = INTERNAL_SERVER_ERROR, description = "an error has occurred", body = ErrorBody),
        ),
    )]
    #[debug_handler(state = AppState)]
    pub async fn find_collection(
        State(collection_service): State<CollectionService>,
        Path(path): Path<FindCollectionPathDto>,
    ) -> Result<Response, CollectionServiceError> {
        match collection_service.find_collection(path).await? {
            Some(result) => Ok((StatusCode::OK, Json(result)).into_response()),
            None => Ok(StatusCode::NOT_FOUND.into_response()),
        }
    }

    /// Create a new collection.
    #[utoipa::path(
        post,
        operation_id = "create-collection",
        tag = "collection",
        path = "/collections",
        responses(
            (status = CREATED, body = CollectionDto),
            (status = INTERNAL_SERVER_ERROR, description = "an error has occurred", body = ErrorBody),
        ),
    )]
    #[debug_handler(state = AppState)]
    pub async fn create_collection(
        State(collection_service): State<CollectionService>,
        Json(body): Json<CreateCollectionBodyDto>,
    ) -> Result<(StatusCode, Json<CollectionDto>), CollectionServiceError> {
        let result = collection_service.create_collection(body).await?;

        Ok((StatusCode::CREATED, Json(result)))
    }

    /// Update a collection.
    #[utoipa::path(
        put,
        operation_id = "update-collection",
        tag = "collection",
        path = "/collections/{identifier}",
        params(
            UpdateCollectionPathDto
        ),
        responses(
            (status = OK, body = CollectionDto),
            (status = NOT_FOUND, description = "the collection does not exist"),
            (status = INTERNAL_SERVER_ERROR, description = "an error has occurred", body = ErrorBody),
        ),
    )]
    #[debug_handler(state = AppState)]
    pub async fn update_collection(
        State(collection_service): State<CollectionService>,
        Path(path): Path<UpdateCollectionPathDto>,
        Json(body): Json<UpdateCollectionBodyDto>,
    ) -> Result<Response, CollectionServiceError> {
        match collection_service.update_collection(path, body).await? {
            Some(result) => Ok((StatusCode::OK, Json(result)).into_response()),
            None => Ok(StatusCode::NOT_FOUND.into_response()),
        }
    }

    /// Remove a collection.
    #[utoipa::path(
        delete,
        operation_id = "delete-collection",
        tag = "collection",
        path = "/collections/{identifier}",
        params(
            RemoveCollectionPathDto
        ),
        responses(
            (status = OK, body = CollectionDto),
            (status = NOT_FOUND, description = "the collection does not exist"),
            (status = INTERNAL_SERVER_ERROR, description = "an error has occurred", body = ErrorBody),
        ),
    )]
    #[debug_handler(state = AppState)]
    pub async fn remove_collection(
        State(collection_service): State<CollectionService>,
        Path(path): Path<RemoveCollectionPathDto>,
    ) -> Result<Response, CollectionServiceError> {
        match collection_service.remove_collection(path).await? {
            Some(result) => Ok((StatusCode::OK, Json(result)).into_response()),
            None => Ok(StatusCode::NOT_FOUND.into_response()),
        }
    }
}
