use bollard::image::ListImagesOptions;
use bollard::image::RemoveImageOptions;
use bollard::models::ImageSummary;
use bollard::Docker;
use bollard::API_DEFAULT_VERSION;
use log::info;
use std::collections::HashMap;

use crate::DateArgs;

pub struct DockerActions {
    docker: Docker,
    repository: Option<String>,
    tags: Vec<String>,
    date: DateArgs,
}

impl DockerActions {
    pub fn new(
        socket: String,
        repository: Option<String>,
        tags: Vec<String>,
        date: DateArgs,
    ) -> Result<Self, bollard::errors::Error> {
        let docker = Docker::connect_with_socket(&socket, 120, API_DEFAULT_VERSION)?;

        Ok(Self {
            docker,
            repository,
            tags,
            date,
        })
    }

    pub async fn get(&self) -> Result<Vec<ImageSummary>, bollard::errors::Error> {
        let mut image_filters = HashMap::new();

        // why using &self.repository instead of selft.repository ?
        if let Some(r) = &self.repository {
            image_filters.insert("reference", vec![r.as_str()]);
        }

        self.docker
            .list_images(Some(ListImagesOptions {
                all: true,
                filters: image_filters,
                ..Default::default()
            }))
            .await
    }

    pub async fn delete(
        &self,
        images: Vec<ImageSummary>,
        force: bool,
        dry_run: bool,
    ) -> Result<i64, bollard::errors::Error> {
        let mut removed_size = 0;
        for image in images {
            info!("deleting: {}", image.id);

            if !dry_run {
                let res = self
                    .docker
                    .remove_image(
                        &image.id,
                        Some(RemoveImageOptions {
                            force,
                            ..Default::default()
                        }),
                        None,
                    )
                    .await;

                if let Err(e) = res {
                    return Err(e);
                }
            }

            removed_size += image.size;
        }

        Ok(removed_size)
    }

    pub fn filter(&self, images: Vec<ImageSummary>) -> Vec<ImageSummary> {
        let mut to_be_deleted: Vec<ImageSummary> = vec![];

        for image in images {
            if self
                .date
                .stop
                .map_or(self.date.start > image.created, |stop| {
                    self.date.start > image.created && image.created > stop
                })
                && image.repo_tags.iter().any(|tag| {
                    !tag.contains("ghcr.io/datahearth/clear-docker-images")
                        && !tag.contains("datahearth/clear-docker-images")
                        && !self
                            .tags
                            .iter()
                            .any(|excluded_tag| tag.contains(excluded_tag))
                })
            {
                to_be_deleted.push(image);
            }
        }

        return to_be_deleted;
    }
}
