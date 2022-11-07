// @generated automatically by Diesel CLI.

diesel::table! {
    packages (id) {
        id -> Integer,
        name -> Tinytext,
        version -> Tinytext,
        readme -> Longtext,
        license -> Longtext,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}
