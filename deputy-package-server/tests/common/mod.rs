#![allow(dead_code)]

use actix::Actor;
use actix_http::{HttpMessage, Request};
use actix_web::web::{Bytes, Data};
use anyhow::Result;
use deputy_package_server::{
    middleware::authentication::local_token::UserToken, test::database::MockDatabase, AppState,
};
use std::rc::Rc;
use tempfile::TempDir;

pub trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

pub fn setup_package_server() -> Result<(TempDir, Data<AppState<MockDatabase>>)> {
    let package_folder = TempDir::new()?;
    let package_folder_string = package_folder.path().to_str().unwrap().to_string();

    let database_address = MockDatabase::default().start();
    Ok((
        package_folder,
        Data::new(AppState {
            package_folder: package_folder_string,
            database_address,
        }),
    ))
}

pub fn set_mock_user_token(request: &Request) {
    request
        .extensions_mut()
        .insert::<Rc<UserToken>>(Rc::new(UserToken {
            id: "test-id".to_string(),
            email: "test-email".to_string(),
        }));
}
