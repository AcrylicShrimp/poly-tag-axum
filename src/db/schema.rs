// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tag_value_type"))]
    pub struct TagValueType;
}

diesel::table! {
    files (uuid) {
        uuid -> Uuid,
        name -> Text,
        mime -> Text,
        size -> Int8,
        hash -> Int8,
        uploaded_at -> Timestamp,
    }
}

diesel::table! {
    stagings (uuid) {
        uuid -> Uuid,
        staged_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TagValueType;

    tag_templates (uuid) {
        uuid -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        value_type -> Nullable<TagValueType>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    tags (uuid) {
        uuid -> Uuid,
        template_uuid -> Uuid,
        file_uuid -> Uuid,
        value_string -> Nullable<Text>,
        value_integer -> Nullable<Int8>,
        value_boolean -> Nullable<Bool>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(tags -> files (file_uuid));
diesel::joinable!(tags -> tag_templates (template_uuid));

diesel::allow_tables_to_appear_in_same_query!(
    files,
    stagings,
    tag_templates,
    tags,
);
