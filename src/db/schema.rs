// @generated automatically by Diesel CLI.

diesel::table! {
    uploads (uuid) {
        uuid -> Uuid,
        file_name -> Nullable<Varchar>,
        uploaded_size -> Int8,
        uploaded_at -> Timestamp,
    }
}
