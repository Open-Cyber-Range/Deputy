mod common;

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use anyhow::Result;
    use deputy_library::test::create_test_package;
    use deputy_package_server::routes::package::add_package;

    #[actix_web::test]
    async fn test_adding_package() -> Result<()> {
        let app = test::init_service(App::new().service(add_package)).await;

        let test_package = create_test_package()?;
        let payload = Vec::try_from(test_package)?;

        let request = test::TestRequest::put()
            .uri("/package")
            .set_payload(payload)
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        Ok(())
    }
}
