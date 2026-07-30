use crate::api::doc::CUSTOM_FIELD_TAG;
use crate::api::error::{ApiError, ApiResult};
use crate::api::{DeleteBody, ResourceParams, UnpagedResponse, error};
use crate::app::AppState;
use crate::config::Action;
use crate::extract::{Ctx, Json, Path, Query};
use crate::model::custom_field::{CustomField, NewCustomField};
use crate::model::enums::{ResourceProperty, ResourceType};
use crate::resource::custom_field::{CustomFieldInfo, Field};
use crate::schema::custom_field;
use crate::string::SmallString;
use crate::time::DateTime;
use crate::{api, snapshot};
use diesel::{Insertable, OptionalExtension, QueryDsl, RunQueryDsl, SaveChangesDsl};
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(get, update, delete))
}

/// Lists all custom fields.
///
/// Doesn't use paging.
#[utoipa::path(
    get,
    path = "/custom-fields",
    tag = CUSTOM_FIELD_TAG,
    params(ResourceParams),
    responses(
        (status = 200, body = UnpagedResponse<CustomFieldInfo>),
        (status = 403, description = "Privileges are too low"),
    ),
)]
async fn list(
    Ctx(ctx, connection_pool): Ctx,
    Query(resource): Query<ResourceParams<Field>>,
) -> ApiResult<Json<UnpagedResponse<CustomFieldInfo>>> {
    ctx.verify_privilege(Action::CustomFieldList)?;

    connection_pool
        .transaction(move |conn| {
            let all_custom_fields: Vec<CustomField> = custom_field::table
                .order(custom_field::name)
                .load(conn)?;
            CustomFieldInfo::new_batch(conn, all_custom_fields, resource.fields)
                .map(|results| UnpagedResponse { results })
        })
        .await
        .map(Json)
}

/// Retrieves information about an existing custom field.
#[utoipa::path(
    get,
    path = "/custom-field/{id}",
    tag = CUSTOM_FIELD_TAG,
    params(
        ("id" = i64, Path, description = "Custom Field ID"),
        ResourceParams,
    ),
    responses(
        (status = 200, body = CustomFieldInfo),
        (status = 403, description = "Privileges are too low"),
        (status = 404, description = "Custom field does not exist"),
    ),
)]
async fn get(
    Ctx(ctx, connection_pool): Ctx,
    Path(custom_field_id): Path<i64>,
    Query(params): Query<ResourceParams<Field>>,
) -> ApiResult<Json<CustomFieldInfo>> {
    ctx.verify_privilege(Action::CustomFieldView)?;

    connection_pool
        .transaction(move |conn| {
            let cf: CustomField = custom_field::table
                .find(custom_field_id)
                .first(conn)
                .optional()?
                .ok_or(ApiError::NotFound(ResourceType::CustomField))?;
            CustomFieldInfo::new(conn, cf, params.fields)
                .map(Json)
                .map_err(ApiError::from)
        })
        .await
}

/// Request body for creating a custom field.
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct CustomFieldCreateBody {
    name: SmallString,
    description: Option<SmallString>,
    color: Option<SmallString>,
}

/// Creates a new custom field.
#[utoipa::path(
    post,
    path = "/custom-fields",
    tag = CUSTOM_FIELD_TAG,
    params(ResourceParams),
    request_body = CustomFieldCreateBody,
    responses(
        (status = 200, body = CustomFieldInfo),
        (status = 403, description = "Privileges are too low"),
        (status = 409, description = "A custom field with this name already exists"),
    ),
)]
async fn create(
    Ctx(ctx, connection_pool): Ctx,
    Query(params): Query<ResourceParams<Field>>,
    Json(body): Json<CustomFieldCreateBody>,
) -> ApiResult<Json<CustomFieldInfo>> {
    ctx.verify_privilege(Action::CustomFieldCreate)?;

    let custom_field = connection_pool
        .transaction(move |conn| {
            let result = NewCustomField {
                name: &body.name,
                description: body.description.as_deref().unwrap_or(""),
                color: body.color.as_deref().unwrap_or(""),
            }
            .insert_into(custom_field::table)
            .get_result(conn);
            let cf = error::map_unique_violation(result, ResourceProperty::CustomFieldName)?;
            snapshot::custom_field::creation_snapshot(conn, ctx.client, &cf)?;
            Ok::<_, ApiError>(cf)
        })
        .await?;
    connection_pool
        .transaction(move |conn| CustomFieldInfo::new(conn, custom_field, params.fields))
        .await
        .map(Json)
}

/// Request body for updating a custom field.
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct CustomFieldUpdateBody {
    version: DateTime,
    name: SmallString,
    #[serde(default)]
    description: Option<SmallString>,
    #[serde(default)]
    color: Option<SmallString>,
}

/// Updates an existing custom field.
#[utoipa::path(
    put,
    path = "/custom-field/{id}",
    tag = CUSTOM_FIELD_TAG,
    params(
        ("id" = i64, Path, description = "Custom Field ID"),
        ResourceParams,
    ),
    request_body = CustomFieldUpdateBody,
    responses(
        (status = 200, body = CustomFieldInfo),
        (status = 403, description = "Privileges are too low"),
        (status = 404, description = "Custom field does not exist"),
        (status = 409, description = "Version is outdated"),
        (status = 409, description = "A custom field with this name already exists"),
    ),
)]
async fn update(
    Ctx(ctx, connection_pool): Ctx,
    Path(custom_field_id): Path<i64>,
    Query(params): Query<ResourceParams<Field>>,
    Json(body): Json<CustomFieldUpdateBody>,
) -> ApiResult<Json<CustomFieldInfo>> {
    let updated_cf = connection_pool
        .transaction(move |conn| {
            let old_cf: CustomField = custom_field::table
                .find(custom_field_id)
                .first(conn)
                .optional()?
                .ok_or(ApiError::NotFound(ResourceType::CustomField))?;
            api::verify_version(old_cf.last_edit_time, body.version)?;

            ctx.verify_privilege(Action::CustomFieldEditName)?;

            let mut new_cf = old_cf.clone();
            new_cf.name = body.name;
            if let Some(description) = body.description {
                new_cf.description = description;
            }
            if let Some(color) = body.color {
                new_cf.color = color;
            }
            new_cf.last_edit_time = DateTime::now();

            let new_cf: CustomField =
                error::map_unique_violation(new_cf.save_changes(conn), ResourceProperty::CustomFieldName)?;
            snapshot::custom_field::modification_snapshot(conn, ctx.client, &old_cf, &new_cf)?;
            Ok::<_, ApiError>(new_cf)
        })
        .await?;
    connection_pool
        .transaction(move |conn| CustomFieldInfo::new(conn, updated_cf, params.fields))
        .await
        .map(Json)
}

/// Deletes a custom field.
#[utoipa::path(
    delete,
    path = "/custom-field/{id}",
    tag = CUSTOM_FIELD_TAG,
    params(
        ("id" = i64, Path, description = "Custom Field ID"),
    ),
    request_body = DeleteBody,
    responses(
        (status = 200, body = ()),
        (status = 403, description = "Privileges are too low"),
        (status = 404, description = "Custom field does not exist"),
        (status = 409, description = "Version is outdated"),
    ),
)]
async fn delete(
    Ctx(ctx, connection_pool): Ctx,
    Path(custom_field_id): Path<i64>,
    Json(client_version): Json<DeleteBody>,
) -> ApiResult<Json<()>> {
    ctx.verify_privilege(Action::CustomFieldDelete)?;

    connection_pool
        .transaction(move |conn| {
            let cf: CustomField = custom_field::table
                .find(custom_field_id)
                .first(conn)
                .optional()?
                .ok_or(ApiError::NotFound(ResourceType::CustomField))?;
            api::verify_version(cf.last_edit_time, *client_version)?;
            snapshot::custom_field::deletion_snapshot(conn, ctx.client, &cf)?;
            diesel::delete(custom_field::table.find(custom_field_id)).execute(conn)?;
            Ok::<_, ApiError>(Json(()))
        })
        .await
}
