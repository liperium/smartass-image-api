// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "image_function"))]
    pub struct ImageFunction;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ImageFunction;

    image_path (id) {
        id -> Int4,
        filename -> Varchar,
        task_id -> Varchar,
        user_id -> Varchar,
        function -> ImageFunction,
    }
}
