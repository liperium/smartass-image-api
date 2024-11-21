use diesel::prelude::*;
use diesel_derive_enum::DbEnum;

#[derive(DbEnum, Debug)]
#[ExistingTypePath = "crate::schema::sql_types::ImageFunction"]
pub enum ImageFunction {
    Help,
    Proof,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::image_path)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ImagePath {
    pub id: i32,
    pub filename: String,
    pub task_id: String,
    pub user_id: String,
    pub function: ImageFunction,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::image_path)]
pub struct NewImagePath<'a> {
    pub filename: &'a str,
    pub task_id: String,
    pub user_id: String,
    pub function: ImageFunction,
}
