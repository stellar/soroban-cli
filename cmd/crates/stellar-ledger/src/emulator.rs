use crate::docker::DockerConnection;

pub enum Error {}

pub async fn run() -> Result<(), Error> {
    let d = DockerConnection::new().await;
    let zondax_speculos_image =
        "docker.io/zondax/builder-zemu:speculos-3a3439f6b45eca7f56395673caaf434c202e7005";
    d.get_image_with_defaults(zondax_speculos_image)
        .await
        .unwrap();

    let container_id = d
        .get_container_with_defaults(zondax_speculos_image)
        .await
        .unwrap();

    // This is starting up, but i think it fails pretty quickly, and i think we have it configured to delete itself once it starts. yep, when auto_remove is set to false, it sticks around but it exits right away
    d.start_container_with_defaults(&container_id)
        .await
        .unwrap();

    d.stream_logs(&container_id).await;
    Ok(())
}



// next steps:
// have this docker connection start the speculos emulator
// see if i can use that emulator in tests like they do with zemu
