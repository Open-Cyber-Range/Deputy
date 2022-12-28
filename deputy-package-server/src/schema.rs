// @generated automatically by Diesel CLI.

diesel::table! {
    packages (id) {
        id -> Binary,
        name -> Tinytext,
        version -> Tinytext,
        license -> Text,
        readme -> Mediumtext,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}
