use crate::packages;
use crate::utils;
use anyhow::Result;
use brane_dsl::indexes::PackageIndex;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use reqwest::{self, multipart::Form, multipart::Part, Body, Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JValue};
use specifications::package::PackageInfo;
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use tar::Archive;
use tokio::fs::File as TokioFile;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub id: i32,
    // Metadata
    pub created: String,
    pub kind: String,
    pub name: String,
    pub uploaded: String,
    pub uuid: String,
    pub version: String,
    // Content
    pub description: Option<String>,
    pub functions_json: Option<String>,
    pub types_json: Option<String>,
    // File
    pub checksum: i64,
    pub filename: String,
}

///
///
///
pub fn login(
    _host: String,
    _username: String,
) -> Result<()> {
    unimplemented!()
}

///
///
///
pub fn logout(_host: String) -> Result<()> {
    unimplemented!()
}

///
///
///
pub async fn pull(
    name: String,
    version: Option<String>,
) -> Result<()> {
    let version = version.expect("please provide version");

    let url = format!("http://127.0.0.1:8080/packages/{}/{}", name, version);
    let package: Result<Package, _> = reqwest::get(&url).await?.json().await;
    if package.is_err() {
        println!("Cannot find version '{}' of package '{}'", version, name);
        return Ok(());
    }

    let url = format!("http://127.0.0.1:8080/packages/{}/{}/archive", name, version);
    let mut package_archive = reqwest::get(&url).await?;
    let package = package.unwrap();

    // Write package archive to temporary file
    let temp_filepath = env::temp_dir().join(package.filename);
    let mut temp_file = File::create(&temp_filepath)?;
    while let Some(chunk) = package_archive.chunk().await? {
        temp_file.write_all(&chunk)?; // If causes bug, use .write(&chunk)
    }

    // Verify checksum
    let checksum = utils::calculate_crc32(&temp_filepath)?;
    if checksum != package.checksum as u32 {
        println!("Download failed, checksums don't match!");
        return Ok(());
    }

    // Unpack temporary file to target location
    let archive_file = File::open(temp_filepath)?;
    let package_dir = packages::get_package_dir(&name, Some(&version))?;
    fs::create_dir_all(&package_dir)?;

    let mut archive = Archive::new(GzDecoder::new(archive_file));
    archive.unpack(&package_dir)?;

    Ok(())
}

///
///
///
pub async fn push(
    name: String,
    version: String,
) -> Result<()> {
    let package_dir = packages::get_package_dir(&name, Some(&version))?;
    let archive_filename = "archive.tar.gz";
    let archive_filepath = env::temp_dir().join(archive_filename);
    let archive_file = File::create(&archive_filepath)?;

    // Create package tarball
    let gz = GzEncoder::new(&archive_file, Compression::fast());
    let mut tar = tar::Builder::new(gz);
    tar.append_dir_all(".", package_dir)?;
    tar.into_inner()?;

    // Calcualte checksum
    let checksum = utils::calculate_crc32(&archive_filepath)?;

    // Upload file
    let url = format!(
        "http://127.0.0.1:8080/packages/{}/{}?checksum={}",
        name, version, checksum
    );
    let request = Client::new().request(Method::POST, &url);

    let file = TokioFile::open(&archive_filepath).await?;
    let reader = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));

    let mut form = Form::new();
    form = form.part("file", Part::stream(reader).file_name(archive_filename));

    let request = request.multipart(form);
    let response = request.send().await?;

    println!("{:?}", response.text().await?);

    Ok(())
}

///
///
///
pub async fn search(term: String) -> Result<()> {
    let url = format!("http://127.0.0.1:8080/packages?t={}", term);
    let packages: Vec<Package> = reqwest::get(&url).await?.json().await?;

    for package in packages {
        println!("{}", package.name);
    }

    Ok(())
}

///
///
///
pub async fn get_package_index(online: bool) -> Result<PackageIndex> {
    let packages: JValue = if online {
        let url = "http://127.0.0.1:8080/packages";
        reqwest::get(url).await?.json().await?
    } else {
        let packages_dir = packages::get_packages_dir();
        if !packages_dir.exists() {
            return Ok(PackageIndex::empty());
        }

        let mut package_infos = Vec::<PackageInfo>::new();

        let packages = fs::read_dir(packages_dir)?;
        for package in packages {
            let package_path = package?.path();
            if !package_path.is_dir() {
                continue;
            }

            let versions = fs::read_dir(package_path)?;
            for version in versions {
                let path = version?.path();
                let package_file = path.join("package.yml");

                if let Ok(package_info) = PackageInfo::from_path(package_file) {
                    package_infos.push(package_info);
                }
            }
        }

        json!(package_infos)
    };

    PackageIndex::from_value(packages)
}
