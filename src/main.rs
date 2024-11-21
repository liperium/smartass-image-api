#[macro_use]
extern crate rocket;
extern crate diesel;
extern crate reqwest;
extern crate rocket_multipart_form_data;

use diesel::RunQueryDsl;
use hdp_web_server::models::{ImageFunction, NewImagePath};
use hdp_web_server::{establish_connection, get_filename};
use reqwest::header::AUTHORIZATION;
use rocket::fs::NamedFile;
use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;
use rocket::Data;
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use serde_json::Value;
use std::fs::File;
use std::path::{Path, PathBuf};

const AUTH0_USERINFO_URL: &str = ""/*Auth0 UserInfo projects domain*/;

async fn get_user_id(access_token: &str) -> Result<String, Custom<String>> {
    let client_builder = reqwest::Client::builder(); // Necessary because we running in a minimal docker container

    let Ok(client) = client_builder.build() else {
        return Err(Custom(
            Status::InternalServerError,
            "Couldn't bind TLS".to_owned(),
        ));
    };

    let res = client
        .get(AUTH0_USERINFO_URL)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await;
    let Ok(response) = res else {
        return Err(Custom(
            Status::InternalServerError,
            "Couldn't reach auth server".to_owned(),
        ));
    };
    let body = response.text().await.unwrap_or_default();
    if body == "Unauthorized" {
        return Err(Custom(
            Status::Unauthorized,
            "Auth0 token is bad".to_owned(),
        ));
    }
    return match extract_auth0_id(body.as_str()) {
        Some(extracted_id) => Ok(extracted_id),
        None => Err(Custom(
            Status::BadRequest,
            "Can't extract user id".to_owned(),
        )),
    };
}

fn extract_auth0_id(auth0_response: &str) -> Option<String> {
    println!("{}", auth0_response);
    let parsed_json: Result<Value, _> = serde_json::from_str(auth0_response);
    if let Ok(parsed_json) = parsed_json {
        if let Some(sub_value) = parsed_json.get("sub") {
            if let Some(sub_str) = sub_value.as_str() {
                if let Some(sub) = sub_str.split('|').nth(1) {
                    return Some(sub.to_string());
                }
            } else {
                println!("'sub' value is not a string");
            }
        } else {
            println!("'sub' key not found in JSON");
        }
    } else {
        println!("Failed to parse JSON");
    }
    None
}

#[post("/upload_image", data = "<data>")]
async fn upload(content_type: &ContentType, data: Data<'_>) -> Result<String, Custom<String>> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("user_id"),
        MultipartFormDataField::file("image")
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .unwrap(),
        MultipartFormDataField::text("task_type"), // Add other fields as needed
        MultipartFormDataField::text("task_id"),
    ]);

    let multipart_form_data: MultipartFormData =
        match MultipartFormData::parse(content_type, data, options).await {
            Ok(data) => data,
            Err(_) => return Err(Custom(Status::BadRequest, "Bad request format".to_string())),
        };
    let mut task_id: String = "-1".to_string();
    let mut task_type: i32 = -1;
    let mut user_id: String = "-1".to_string();
    for (field_name, field_values) in multipart_form_data.texts.iter() {
        match &(**field_name) {
            "task_id" => {
                task_id = field_values[0].text.clone();
            }
            "task_type" => {
                task_type = field_values[0].text.parse::<i32>().unwrap();
            }
            "user_id" => {
                user_id = field_values[0].text.clone();
            }
            _ => {}
        }
    }
    // Check if all required fields are present, should be because of multipart parse above
    if (task_id.eq("-1")) || (task_type == -1) || user_id.eq("-1") {
        return Err(Custom(
            Status::BadRequest,
            "Missing required fields".to_string(),
        ));
    }

    //let user_id = get_user_id(auth_token.as_str()).await;

    if let Some(file_fields) = multipart_form_data.files.get("image") {
        let file_path = &file_fields[0].path;

        let filename = get_filename(&task_id, &user_id, task_type);
        let image_path = NewImagePath {
            filename: &filename,
            task_id,
            user_id: user_id.clone(),
            function: ImageFunction::Help,
        };

        // Open the temporary file.
        let mut temp_file = File::open(file_path).expect("Failed to open multiparts temp file");

        // Create a new file to save the image.
        let mut new_file = File::create(Path::new("./images").join(image_path.filename))
            .expect("Failed to create new file");

        // Copy the contents of the temporary file to the new file.
        std::io::copy(&mut temp_file, &mut new_file).expect("Failed to copy file");
        // Save in database
        let connection = &mut establish_connection();
        diesel::insert_into(hdp_web_server::schema::image_path::table)
            .values(&image_path)
            .execute(connection)
            .expect("Can't push to database");

        return Ok(user_id);
    } else {
        return Err(Custom(
            Status::BadRequest,
            "No image file found".to_string(),
        ));
    }
}

#[post("/get_image", data = "<data>")]
async fn get_image(
    content_type: &ContentType,
    data: Data<'_>,
) -> Result<NamedFile, Custom<String>> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("user_id"),
        MultipartFormDataField::text("task_id"),
        MultipartFormDataField::text("task_type"), // Add other fields as needed
    ]);

    let multipart_form_data = match MultipartFormData::parse(content_type, data, options).await {
        Ok(data) => data,
        Err(_) => return Err(Custom(Status::BadRequest, "Bad request format".to_string())),
    };
    let mut task_id: String = "-1".to_string();
    let mut task_type: i32 = -1;
    let mut user_id: String = "-1".to_string();
    for (field_name, field_values) in multipart_form_data.texts.iter() {
        match &**field_name {
            "task_id" => {
                task_id = field_values[0].text.clone();
            }
            "task_type" => {
                task_type = field_values[0].text.parse::<i32>().unwrap();
            }
            "user_id" => {
                user_id = field_values[0].text.clone();
            }
            _ => {}
        }
    }
    // Check if all required fields are present, should be because of multipart parse above
    if (task_id.eq("-1")) || (task_type == -1) || user_id.eq("-1") {
        return Err(Custom(
            Status::BadRequest,
            "Missing required fields".to_string(),
        ));
    }
    //let user_id = get_user_id(&auth_token).await;

    let filename = get_filename(&task_id, &user_id, task_type);
    // TODO database check??

    println!("Getting file : {}", filename);
    let path = Path::new("./images").join(filename);

    return match NamedFile::open(path).await.ok() {
        Some(file) => Ok(file),
        None => Err(Custom(
            Status::NotFound,
            "Requested file not found".to_owned(),
        )),
    };
}
#[get("/")]
fn index() -> &'static str {
    "API is running!"
}

#[launch]
fn rocket() -> _ {
    // Tests remote DB connection
    establish_connection();
    rocket::build().mount("/", routes![index, image, upload, get_image])
}

#[get("/image/<file..>")]
async fn image(file: PathBuf) -> Option<NamedFile> {
    let path = Path::new("./").join(file);
    NamedFile::open(path).await.ok()
}
