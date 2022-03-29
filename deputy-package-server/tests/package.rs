mod common;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use actix_web::{test, App};
    use anyhow::Result;
    use deputy_library::test::create_test_package;
    use deputy_package_server::{
        routes::package::add_package,
        test::{
            create_predictable_temporary_folders, create_test_app_state, generate_random_string,
        },
    };

    #[actix_web::test]
    async fn test_adding_package() -> Result<()> {
        let random_string = generate_random_string(8)?;
        let (package_folder, _) = create_predictable_temporary_folders(random_string.clone())?;
        let app_state = create_test_app_state(random_string)?;
        let app = test::init_service(App::new().app_data(app_state).service(add_package)).await;

        let test_package = create_test_package()?;
        let package_name = test_package.metadata.name.clone();
        let payload = Vec::try_from(test_package)?;

        let request = test::TestRequest::put()
            .uri("/package")
            .set_payload(payload)
            .to_request();
        let response = test::call_service(&app, request).await;

        assert!(response.status().is_success());
        assert!(PathBuf::from(package_folder).join(package_name).exists());
        Ok(())
    }
}
