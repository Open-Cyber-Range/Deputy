mod common;

#[cfg(test)]
mod tests {
    use super::common::BodyTest;
    use actix_http::Payload;
    use actix_web::{body::to_bytes, test, web::Data, App};
    use anyhow::Result;
    use deputy_library::{
        package::PackageStream,
        test::{create_test_package, generate_random_string},
    };
    use deputy_package_server::{
        routes::package::add_package,
        routes::package::download_package,
        test::{create_predictable_temporary_folders, create_test_app_state},
        AppState,
    };
    use std::path::PathBuf;
    use std::str::from_utf8;

    fn setup_package_server() -> Result<(String, Data<AppState>)> {
        let randomizer = generate_random_string(10)?;
        let (package_folder, _) = create_predictable_temporary_folders(randomizer.clone())?;
        let app_state = create_test_app_state(randomizer)?;
        Ok((package_folder, app_state))
    }

    #[actix_web::test]
    async fn successfully_add_package() -> Result<()> {
        let (package_folder, app_state) = setup_package_server()?;
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

    // #[actix_web::test]
    // async fn send_package_via_streaming() -> Result<()> {
    //     let (package_folder, app_state) = setup_package_server()?;
    //     let app = test::init_service(
    //         App::new()
    //             .app_data(app_state)
    //             .service(add_package_streaming),
    //     )
    //     .await;

    //     let test_package = create_test_package()?;
    //     let package_name = test_package.metadata.name.clone();

    //     let stream: PackageStream = test_package.try_into()?;
    //     let request = test::TestRequest::put().uri("/package/stream").to_request();
    //     let (request, _) = request.replace_payload(Payload::from(stream));
    //     let response = test::call_service(&app, request).await;

    //     assert!(response.status().is_success());
    //     assert!(PathBuf::from(package_folder).join(package_name).exists());
    //     Ok(())
    // }

    #[actix_web::test]
    async fn send_invalid_package_bytes() -> Result<()> {
        let (_, app_state) = setup_package_server()?;
        let app = test::init_service(App::new().app_data(app_state).service(add_package)).await;

        let payload = vec![0, 1, 1, 1];
        let request = test::TestRequest::put()
            .uri("/package")
            .set_payload(payload)
            .to_request();
        let response = test::call_service(&app, request).await;

        assert!(response.status().is_client_error());
        let body = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body.as_str(), "Failed to parse package bytes");
        Ok(())
    }

    #[actix_web::test]
    async fn send_invalid_package_metadata() -> Result<()> {
        let (_, app_state) = setup_package_server()?;
        let app = test::init_service(App::new().app_data(app_state).service(add_package)).await;

        let mut test_package = create_test_package()?;
        test_package.metadata.checksum = String::from("ssssss");
        let payload = Vec::try_from(test_package)?;
        let request = test::TestRequest::put()
            .uri("/package")
            .set_payload(payload)
            .to_request();
        let response = test::call_service(&app, request).await;

        assert!(response.status().is_client_error());
        let body = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body.as_str(), "Failed to validate the package");
        Ok(())
    }

    #[actix_web::test]
    async fn submit_package_with_same_version_twice() -> Result<()> {
        let (_, app_state) = setup_package_server()?;
        let app = test::init_service(App::new().app_data(app_state).service(add_package)).await;

        let test_package = create_test_package()?;
        let payload = Vec::try_from(test_package)?;
        let request = test::TestRequest::put()
            .uri("/package")
            .set_payload(payload.clone())
            .to_request();
        test::call_service(&app, request).await;
        let second_request = test::TestRequest::put()
            .uri("/package")
            .set_payload(payload)
            .to_request();
        let second_response = test::call_service(&app, second_request).await;

        assert!(second_response.status().is_client_error());
        let body = to_bytes(second_response.into_body()).await.unwrap();
        assert_eq!(
            body.as_str(),
            "Package version on the server is either same or later"
        );
        Ok(())
    }

    #[actix_web::test]
    async fn download_package_with_name_and_version() -> Result<()> {
        let (_, app_state) = setup_package_server()?;

        let test_package = create_test_package()?;

        let package_name = test_package.metadata.name.clone();
        let package_version = test_package.metadata.version.clone();

        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .service(download_package)
                .service(add_package),
        )
        .await;

        let payload = Vec::try_from(test_package)?;

        let request = test::TestRequest::put()
            .uri("/package")
            .set_payload(payload.clone())
            .to_request();

        test::call_service(&app, request).await;

        let request = test::TestRequest::get()
            .uri(&format!(
                "/package/{}/{}/download",
                package_name, package_version
            ))
            .to_request();

        let body = test::call_and_read_body(&app, request).await;
        let content = from_utf8(&body).unwrap();
        assert_eq!(content, "some content \n");
        Ok(())
    }
}
