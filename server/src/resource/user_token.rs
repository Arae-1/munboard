use crate::model::user::UserToken;
use crate::resource::field::Mask;
use crate::resource::user::MicroUser;
use crate::string::LargeString;
use crate::time::DateTime;
use serde::Serialize;
use serde_with::skip_serializing_none;
use server_macros::non_nullable_options;
use strum::EnumString;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Copy, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum Field {
    Version,
    User,
    Note,
    Enabled,
    ExpirationTime,
    CreationTime,
    LastEditTime,
    LastUsageTime,
    Token,
    Id,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Full,
    PublicOnly,
}

impl From<Field> for u64 {
    fn from(value: Field) -> Self {
        value as u64
    }
}

/// A single user token.
#[non_nullable_options]
#[skip_serializing_none]
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserTokenInfo {
    /// Resource version. See [versioning](#Versioning).
    version: Option<DateTime>,
    /// Internal identifier of the token. Not a secret; addresses update/delete requests.
    id: Option<Uuid>,
    /// The user that owns the token.
    user: Option<MicroUser>,
    /// A note that describes the token.
    note: Option<LargeString>,
    /// Whether the token is still valid for authentication.
    enabled: Option<bool>,
    /// Time when the token expires.
    #[schema(nullable)]
    expiration_time: Option<Option<DateTime>>,
    /// Time the user token was created.
    creation_time: Option<DateTime>,
    /// Time the user token was last edited.
    last_edit_time: Option<DateTime>,
    /// The last time this token was used during a login involving `?bump-login`.
    last_usage_time: Option<DateTime>,
    /// The token value (UUID) used for authentication. Only visible to the
    /// owner; absent on decrypt failures.
    token: Option<Option<String>>,
}

impl UserTokenInfo {
    pub fn new(
        user: MicroUser,
        user_token: UserToken,
        fields: Mask<Field>,
        visibility: Visibility,
        token: Option<String>,
    ) -> Self {
        UserTokenInfo {
            version: fields[Field::Version].then_some(user_token.last_edit_time),
            id: fields[Field::Id].then_some(user_token.id),
            user: fields[Field::User].then_some(user),
            note: fields[Field::Note].then_some(user_token.note),
            enabled: fields[Field::Enabled].then_some(user_token.enabled),
            expiration_time: fields[Field::ExpirationTime].then_some(user_token.expiration_time),
            creation_time: fields[Field::CreationTime].then_some(user_token.creation_time),
            last_edit_time: fields[Field::LastEditTime].then_some(user_token.last_edit_time),
            last_usage_time: fields[Field::LastUsageTime].then_some(user_token.last_usage_time),
            token: match (fields[Field::Token], visibility == Visibility::Full) {
                (true, true) => Some(token),
                _ => None,
            },
        }
    }
}
