// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        #[max_length = 16]
        id -> Binary,
        name -> Tinytext,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    package_categories (package_id, category_id) {
        #[max_length = 16]
        package_id -> Binary,
        #[max_length = 16]
        category_id -> Binary,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    packages (id) {
        #[max_length = 16]
        id -> Binary,
        name -> Tinytext,
        package_type -> Tinytext,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    versions (id) {
        #[max_length = 16]
        id -> Binary,
        #[max_length = 16]
        package_id -> Binary,
        version -> Tinytext,
        description -> Longtext,
        license -> Text,
        readme_html -> Longtext,
        checksum -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(package_categories -> categories (category_id));
diesel::joinable!(package_categories -> packages (package_id));
diesel::joinable!(versions -> packages (package_id));

diesel::allow_tables_to_appear_in_same_query!(categories, package_categories, packages, versions,);
