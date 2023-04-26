pub mod database;

use self::database::MockDatabase;
use crate::{
    routes::{
        basic::{status, version},
        package::{
            add_package, download_file, download_package, get_all_packages, get_all_versions,
            get_package_version,
        },
    },
    AppState,
};
use actix::Actor;
use actix_web::{
    web::{get, put, scope, Data},
    App, HttpServer,
};
use anyhow::{anyhow, Error, Result};
use deputy_library::test::generate_random_string;
use futures::TryFutureExt;
use get_port::{tcp::TcpPort, Ops, Range};
use std::{
    env,
    fs::{create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use tokio::{
    sync::oneshot::{channel, Receiver, Sender},
    time::timeout,
    try_join,
};

pub struct TestPackageServerBuilder {
    host: String,
    package_folder: String,
}

impl TestPackageServerBuilder {
    pub fn try_new() -> Result<Self> {
        let temporary_directory = env::temp_dir();
        let randomizer = generate_random_string(10)?;
        let package_folder: PathBuf =
            temporary_directory.join(format!("test-package-folder-{randomizer}"));
        create_dir_all(&package_folder)?;
        let package_folder = package_folder.to_str().unwrap().to_string();
        let address = "127.0.0.1";
        let sleep_duration = Duration::from_millis(rand::random::<u64>() % 5000);
        thread::sleep(sleep_duration);

        let port = TcpPort::in_range(
            address,
            Range {
                min: 1024,
                max: 65535,
            },
        )
        .ok_or_else(|| anyhow!("Failed to find a free port for the test server"))?;
        let host = format!("{}:{}", address, port);

        Ok(Self {
            host,
            package_folder,
        })
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn package_folder(mut self, package_folder: &str) -> Self {
        self.package_folder = package_folder.to_string();
        self
    }

    pub fn get_package_folder(&self) -> &str {
        &self.package_folder
    }

    pub fn build(&self) -> TestPackageServer {
        TestPackageServer {
            host: self.host.clone(),
            package_folder: self.package_folder.clone(),
            tx: None,
        }
    }
}

pub struct TestPackageServer {
    host: String,
    package_folder: String,
    tx: Option<Sender<()>>,
}

impl TestPackageServer {
    fn initialize(
        host: String,
        package_folder: String,
        tx: Sender<()>,
        rx: Receiver<()>,
    ) -> Result<()> {
        let runtime = actix_rt::System::new();
        runtime.block_on(async {
            let database = MockDatabase::default().start();
            let app_data: AppState<MockDatabase> = AppState {
                package_folder,
                database_address: database,
            };
            try_join!(
                HttpServer::new(move || {
                    let app_data = Data::new(app_data.clone());
                    App::new()
                        .app_data(app_data)
                        .service(status)
                        .service(version)
                        .service(
                            scope("/api").service(
                                scope("/v1").service(
                                    scope("/package")
                                        .service(
                                            scope("/{package_name}")
                                                .route(
                                                    "",
                                                    get().to(get_all_versions::<MockDatabase>),
                                                )
                                                .service(
                                                    scope("/{version}")
                                                        .route(
                                                            "/download",
                                                            get().to(download_package::<
                                                                MockDatabase,
                                                            >),
                                                        )
                                                        .route(
                                                            "/path/{tail:.*}",
                                                            get().to(download_file::<MockDatabase>),
                                                        )
                                                        .route(
                                                            "",
                                                            get().to(get_package_version::<
                                                                MockDatabase,
                                                            >),
                                                        ),
                                                ),
                                        )
                                        .route("", put().to(add_package::<MockDatabase>))
                                        .route("", get().to(get_all_packages::<MockDatabase>)),
                                ),
                            ),
                        )
                })
                .bind(host)
                .unwrap()
                .run()
                .map_err(|error| anyhow!("Failed to start the server: {:?}", error)),
                async move {
                    tx.send(())
                        .map_err(|error| anyhow!("Failed to send message: {:?}", error))?;
                    rx.await.unwrap();
                    Ok::<(), Error>(())
                },
            )
            .unwrap();
        });
        Ok(())
    }

    pub async fn start(mut self) -> Result<()> {
        let host = self.host.clone();
        let package_folder = self.package_folder.clone();

        let (tx, rx) = channel::<()>();
        let (tx1, rx1) = channel::<()>();
        thread::spawn(|| {
            Self::initialize(host, package_folder, tx, rx1).unwrap();
        });
        timeout(Duration::from_secs(3), rx).await??;

        self.tx = Some(tx1);
        Ok(())
    }
}

impl Drop for TestPackageServer {
    fn drop(&mut self) {
        if Path::new(&self.package_folder).is_dir() {
            remove_dir_all(&self.package_folder).unwrap();
        }
        if let Some(tx) = self.tx.take() {
            tx.send(()).unwrap();
        }
    }
}
