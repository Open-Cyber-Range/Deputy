mod common;

#[cfg(test)]
mod tests {

    use crate::common::{set_mock_user_token, setup_package_server, BodyTest};
    use actix_http::Payload;
    use actix_web::{
        body::to_bytes,
        test,
        web::{get, put, scope},
        App,
    };
    use anyhow::Result;
    use deputy_library::{
        package::{Package, PackageStream},
        test::TempArchive,
    };
    use deputy_package_server::{
        routes::package::{add_package, download_file, download_package, yank_version},
        test::{database::MockDatabase, middleware::MockTokenMiddlewareFactory},
    };
    use std::path::PathBuf;

    #[actix_web::test]
    async fn successfully_add_package() -> Result<()> {
        let (package_folder, app_state) = setup_package_server()?;
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/package", put().to(add_package::<MockDatabase>)),
        )
        .await;

        let archive = TempArchive::builder().build()?;
        let test_package: Package = (&archive).try_into()?;
        let package_name = test_package.metadata.name.clone();

        let stream: PackageStream = test_package.to_stream().await?;
        let request = test::TestRequest::put().uri("/package").to_request();
        let (request, _) = request.replace_payload(Payload::from(stream));
        set_mock_user_token(&request);

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        assert!(PathBuf::from(package_folder.path())
            .join(package_name)
            .exists());
        package_folder.close()?;
        Ok(())
    }

    #[actix_web::test]
    async fn send_invalid_package_checksum() -> Result<()> {
        let (package_folder, app_state) = setup_package_server()?;
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/package", put().to(add_package::<MockDatabase>)),
        )
        .await;

        let archive = TempArchive::builder().build()?;
        let mut test_package: Package = (&archive).try_into()?;

        test_package.metadata.checksum = "invalid".to_string();
        let stream: PackageStream = test_package.to_stream().await?;
        let request = test::TestRequest::put().uri("/package").to_request();
        let (request, _) = request.replace_payload(Payload::from(stream));
        set_mock_user_token(&request);

        let response = test::call_service(&app, request).await;

        assert!(response.status().is_client_error());
        let body = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body.as_str(), "Failed to validate the package");

        package_folder.close()?;
        Ok(())
    }

    #[actix_web::test]
    async fn submit_package_with_same_version_twice() -> Result<()> {
        let (package_folder, app_state) = setup_package_server()?;
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/package", put().to(add_package::<MockDatabase>)),
        )
        .await;

        let archive = TempArchive::builder().build()?;
        let test_package: Package = (&archive).try_into()?;

        let stream: PackageStream = test_package.to_stream().await?;
        let request = test::TestRequest::put().uri("/package").to_request();
        let (request, _) = request.replace_payload(Payload::from(stream));
        set_mock_user_token(&request);

        test::call_service(&app, request).await;

        let second_archive = TempArchive::builder().build()?;
        let second_test_package: Package = (&second_archive).try_into()?;
        let second_stream: PackageStream = second_test_package.to_stream().await?;
        let second_request = test::TestRequest::put().uri("/package").to_request();
        let (second_request, _) = second_request.replace_payload(Payload::from(second_stream));
        set_mock_user_token(&second_request);

        let second_response = test::call_service(&app, second_request).await;

        assert!(second_response.status().is_client_error());
        let body = to_bytes(second_response.into_body()).await.unwrap();
        assert_eq!(
            body.as_str(),
            "Package version on the server is either same or later: 1.0.4"
        );

        package_folder.close()?;
        Ok(())
    }

    #[actix_web::test]
    async fn download_package_with_name_and_version() -> Result<()> {
        let (package_folder, app_state) = setup_package_server()?;
        let archive = TempArchive::builder().build()?;
        let test_package: Package = (&archive).try_into()?;

        let package_name = test_package.metadata.name.clone();
        let package_version = test_package.metadata.version.clone();

        let app = test::init_service(
            App::new().app_data(app_state).service(
                scope("/package")
                    .service(
                        scope("/{package_name}").service(
                            scope("/{version}")
                                .route("/download", get().to(download_package::<MockDatabase>)),
                        ),
                    )
                    .route("", put().to(add_package::<MockDatabase>)),
            ),
        )
        .await;
        let stream: PackageStream = test_package.to_stream().await?;
        let request = test::TestRequest::put().uri("/package").to_request();
        let (request, _) = request.replace_payload(Payload::from(stream));
        set_mock_user_token(&request);

        test::call_service(&app, request).await;

        let request = test::TestRequest::get()
            .uri(&format!(
                "/package/{}/{}/download",
                package_name, package_version
            ))
            .to_request();

        let body = test::call_and_read_body(&app, request).await;
        let search_string = b"and we spent 300 manhours on it...";
        assert!(body
            .windows(search_string.len())
            .any(|window| window == search_string));

        package_folder.close()?;
        Ok(())
    }

    #[actix_web::test]
    async fn download_file_from_package() -> Result<()> {
        let (package_folder, app_state) = setup_package_server()?;
        let archive = TempArchive::builder().zero_filetimes(false).build()?;
        let test_package: Package = (&archive).try_into()?;

        let package_name = test_package.metadata.name.clone();
        let package_version = test_package.metadata.version.clone();

        let app = test::init_service(
            App::new().app_data(app_state).service(
                scope("/package")
                    .service(
                        scope("/{package_name}").service(
                            scope("/{version}")
                                .route("/path/{tail:.*}", get().to(download_file::<MockDatabase>)),
                        ),
                    )
                    .route("", put().to(add_package::<MockDatabase>)),
            ),
        )
        .await;
        let stream: PackageStream = test_package.to_stream().await?;
        let request = test::TestRequest::put().uri("/package").to_request();
        let (request, _) = request.replace_payload(Payload::from(stream));
        set_mock_user_token(&request);

        test::call_service(&app, request).await;

        let request = test::TestRequest::get()
            .uri(&format!(
                "/package/{}/{}/path/src/test_file.txt",
                package_name, package_version
            ))
            .to_request();

        let body = test::call_and_read_body(&app, request).await;
        let search_string = b"Mauris elementum non quam laoreet tristique.";
        assert!(body
            .windows(search_string.len())
            .any(|window| window == search_string));

        package_folder.close()?;
        Ok(())
    }

    #[actix_web::test]
    async fn yank_package() -> Result<()> {
        let (package_folder, app_state) = setup_package_server()?;
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/package", put().to(add_package::<MockDatabase>))
                .route(
                    "/package/{package_name}/{version}/yank/{set_yank}",
                    put().to(yank_version::<MockDatabase>),
                )
                .wrap(MockTokenMiddlewareFactory),
        )
        .await;

        let archive = TempArchive::builder().build()?;
        let test_package: Package = (&archive).try_into()?;
        let package_name = test_package.metadata.name.clone();
        let package_version = test_package.metadata.version.clone();
        let stream: PackageStream = test_package.to_stream().await?;
        let request = test::TestRequest::put().uri("/package").to_request();
        let (request, _) = request.replace_payload(Payload::from(stream));
        set_mock_user_token(&request);
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        assert!(PathBuf::from(package_folder.path())
            .join(package_name.clone())
            .exists());
        package_folder.close()?;

        let uri = format!("/package/{}/{}/yank/true", package_name, package_version);
        let request = test::TestRequest::put().uri(uri.as_str()).to_request();
        set_mock_user_token(&request);
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let body = to_bytes(response.into_body()).await.unwrap();
        let search_string = b"is_yanked\":true";
        assert!(body
            .windows(search_string.len())
            .any(|window| window == search_string));

        Ok(())
    }
}
