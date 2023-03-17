// @generated automatically by Diesel CLI.

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

diesel::allow_tables_to_appear_in_same_query!(
    files,
    stagings,
);
