// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Binary,
        name -> Tinytext,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    package_categories (package_id, category_id) {
        package_id -> Binary,
        category_id -> Binary,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    packages (id) {
        id -> Binary,
        name -> Tinytext,
        package_type -> Tinytext,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    tokens (id) {
        #[max_length = 16]
        id -> Binary,
        name -> Tinytext,
        email -> Tinytext,
        token -> Tinytext,
        user_id -> Tinytext,
        created_at -> Timestamp,
        deleted_at -> Timestamp,
    }
}

diesel::table! {
    versions (id) {
        id -> Binary,
        package_id -> Binary,
        version -> Tinytext,
        description -> Longtext,
        license -> Text,
        is_yanked -> Bool,
        readme_html -> Longtext,
        package_size -> Unsigned<Bigint>,
        checksum -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(package_categories -> categories (category_id));
diesel::joinable!(package_categories -> packages (package_id));
diesel::joinable!(versions -> packages (package_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    package_categories,
    packages,
    tokens,
    versions,
);
