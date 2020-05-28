use crate::ExecuteInfo;
use base64;
use bollard::container::{
    Config, CreateContainerOptions, LogOutput, LogsOptions, RemoveContainerOptions, StartContainerOptions,
    WaitContainerOptions,
};
use bollard::errors::Error;
use bollard::image::ImportImageOptions;
use bollard::Docker;
use futures_util::stream::TryStreamExt;
use hyper::Body;
use serde_json;
use std::default::Default;
use tokio::fs::File as TFile;
use tokio::stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead};
use uuid::Uuid;

type FResult<T> = Result<T, failure::Error>;

///
///
///
pub async fn run_and_wait(exec: ExecuteInfo) -> FResult<(String, String)> {
    let docker = Docker::connect_with_local_defaults()?;

    // Import image if a image file was provided
    if let Some(_) = exec.image_file {
        import_image(&docker, &exec).await?;
    }

    // Start container and wait for completion
    let name = create_and_start_container(&docker, &exec).await?;
    &docker
        .wait_container(&name, None::<WaitContainerOptions<String>>)
        .try_collect::<Vec<_>>()
        .await?;

    // Get stdout and stderr logs from container
    let logs_options = Some(LogsOptions {
        stdout: true,
        stderr: true,
        ..Default::default()
    });

    let log_outputs = &docker.logs(&name, logs_options).try_collect::<Vec<LogOutput>>().await?;

    let mut stderr = String::new();
    let mut stdout = String::new();

    for log_output in log_outputs {
        match log_output {
            LogOutput::StdErr { message } => stderr = message.clone(),
            LogOutput::StdOut { message } => stdout = message.clone(),
            _ => unreachable!(),
        }
    }

    // Don't leave behind any waste: remove container
    let remove_options = Some(RemoveContainerOptions {
        force: true,
        ..Default::default()
    });

    &docker.remove_container(&name, remove_options).await?;

    Ok((stdout, stderr))
}

///
///
///
async fn create_and_start_container(
    docker: &Docker,
    exec: &ExecuteInfo,
) -> FResult<String> {
    // Generate unique (temporary) container name
    let name = Uuid::new_v4().to_string().chars().take(8).collect::<String>();

    let create_options = CreateContainerOptions { name: &name };
    let payload = base64::encode(serde_json::to_string(&exec.payload)?);
    let command = vec![String::from("exec"), payload];

    let create_config = Config {
        image: Some(exec.image.clone()),
        cmd: Some(command),
        ..Default::default()
    };

    docker.create_container(Some(create_options), create_config).await?;
    docker
        .start_container(&name, None::<StartContainerOptions<String>>)
        .await?;

    Ok(name)
}

///
///
///
async fn import_image(
    docker: &Docker,
    exec: &ExecuteInfo,
) -> FResult<()> {
    let image_file = &exec.image_file.clone().unwrap();

    // Abort, if image is already loaded
    if let Ok(_) = docker.inspect_image(&exec.image).await {
        return Ok(());
    }

    let options = ImportImageOptions { quiet: true };

    let file = TFile::open(image_file).await?;
    let byte_stream = FramedRead::new(file, BytesCodec::new()).map(|r| {
        let bytes = r.unwrap().freeze();
        Ok::<_, Error>(bytes)
    });

    let body = Body::wrap_stream(byte_stream);
    docker.import_image(options, body, None).try_collect::<Vec<_>>().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value as JValue};
    use std::path::PathBuf;
    use tokio;

    #[tokio::test]
    async fn name() {
        let image = String::from("arithmetic:1.0.0");
        let image_file = PathBuf::from("./resources/arithmetic.tar");
        let payload = json!({
            "identifier": "1+1",
            "action": "add",
            "arguments": {
                "a": 1,
                "b": 1,
            },
        });

        let exec_info = ExecuteInfo::new(image, Some(image_file), payload);
        let (stdout, _) = run_and_wait(exec_info).await.unwrap();

        let output: JValue = serde_json::from_str(&stdout).unwrap();
        assert_eq!(output["c"].as_i64().unwrap(), 2);
    }
}
