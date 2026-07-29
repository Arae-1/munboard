use crate::api::doc::CUSTOM_FIELD_TAG;
use crate::api::error::{ApiError, ApiResult};
use crate::api::{DeleteBody, PagedResponse, ResourceParams};
use crate::app::AppState;
use crate::config::Action;
use crate::extract::{Ctx, Json, Path, Query};
use crate::model::custom_field::{CustomField, NewCustomField};
use crate::model::enums::{ResourceProperty, ResourceType};
use crate::resource::custom_field::{CustomFieldInfo, Field};
use crate::schema::custom_field;
use crate::snapshot::custom_field::SnapshotData;
use crate::string::SmallString;
use crate::time::DateTime;
use crate::{api, snapshot};
use diesel::dsl::exists;
use diesel::{ExpressionMethods, Insertable, OptionalExtension, QueryDsl, RunQueryDsl};
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(create))
        .routes(routes!(get, update, delete))
}

/// Lists all custom fields.
#[utoipa::path(
    get,
    path = "/custom-fields",
    tag = CUSTOM_FIELD_TAG,
    params(ResourceParams),
    responses(
        (status = 200, body = PagedResponse<CustomFieldInfo>),
        (status = 403, description = "Privileges are too low"),
    ),
)]
async fn list(
    Ctx(ctx, connection_pool): Ctx,
    Query(resource): Query<ResourceParams<Field>>,
) -> ApiResult<Json<PagedResponse<CustomFieldInfo>>> {
    ctx.verify_privilege(Action::CustomFieldList)?;

    connection_pool
        .transaction(move |conn| {
            let all_fields: Vec<CustomField> = custom_field::table
                .order(custom_field::name)
                .load(conn)?;
            let total = all_fields.len() as i64;
            let results = CustomFieldInfo::new_batch(conn, &ctx, all_fields, resource.fields)?;
            Ok::<_, ApiError>(Json(PagedResponse {
                query: resource.query,
                offset: 0,
                limit: total,
                total,
                results,
            }))
        })
        .await
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
            let cf_exists: bool =
                diesel::select(exists(custom_field::table.find(custom_field_id))).first(conn)?;
            if !cf_exists {
                return Err(ApiError::NotFound(ResourceType::CustomField));
            }
            CustomFieldInfo::new_from_id(conn, &ctx, custom_field_id, params.fields)
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
    path = "/custom-field",
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

    let cf = connection_pool
        .transaction({
            move |conn| {
                let result = NewCustomField {
                    name: &body.name,
                    description: body.description.as_deref().unwrap_or(""),
                    color: body.color.as_deref(),
                }
                .insert_into(custom_field::table)
                .get_result(conn);
                let custom_field: CustomField =
                    api::error::map_unique_violation(result, ResourceProperty::CustomFieldName)?;

                let cf_data = SnapshotData::retrieve(custom_field.clone());
                snapshot::custom_field::creation_snapshot(conn, ctx.client, custom_field.id, cf_data)?;
                Ok::<_, ApiError>(custom_field)
            }
        })
        .await?;
    connection_pool
        .transaction(move |conn| CustomFieldInfo::new(conn, &ctx, cf, params.fields))
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
    color: Option<Option<SmallString>>,
}

/// Updates an existing custom field (rename).
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
    let cf_id = connection_pool
        .transaction({
            let ctx = ctx.clone();
            move |conn| {
                let old_cf: CustomField = custom_field::table
                    .find(custom_field_id)
                    .first(conn)
                    .optional()?
                    .ok_or(ApiError::NotFound(ResourceType::CustomField))?;
                api::verify_version(old_cf.creation_time, body.version)?;

                let old_snapshot_data = SnapshotData::retrieve(old_cf.clone());
                ctx.verify_privilege(Action::CustomFieldEditName)?;

                let result = diesel::update(custom_field::table.find(custom_field_id))
                    .set(custom_field::name.eq(&body.name))
                    .execute(conn);
                api::error::map_unique_violation::<usize>(result, ResourceProperty::CustomFieldName)?;

                if let Some(description) = &body.description {
                    diesel::update(custom_field::table.find(custom_field_id))
                        .set(custom_field::description.eq(description))
                        .execute(conn)?;
                }
                if let Some(color) = &body.color {
                    diesel::update(custom_field::table.find(custom_field_id))
                        .set(custom_field::color.eq(color.as_deref()))
                        .execute(conn)?;
                }

                let updated_cf: CustomField = custom_field::table.find(custom_field_id).first(conn)?;
                let new_snapshot_data = SnapshotData::retrieve(updated_cf.clone());

                snapshot::custom_field::modification_snapshot(
                    conn,
                    ctx.client,
                    custom_field_id,
                    old_snapshot_data,
                    new_snapshot_data,
                )?;
                Ok::<_, ApiError>(custom_field_id)
            }
        })
        .await?;
    connection_pool
        .transaction(move |conn| CustomFieldInfo::new_from_id(conn, &ctx, cf_id, params.fields))
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
            api::verify_version(cf.creation_time, *client_version)?;

            let cf_data = SnapshotData::retrieve(cf);
            snapshot::custom_field::deletion_snapshot(conn, ctx.client, custom_field_id, cf_data)?;

            diesel::delete(custom_field::table.find(custom_field_id)).execute(conn)?;
            Ok::<_, ApiError>(Json(()))
        })
        .await
}
