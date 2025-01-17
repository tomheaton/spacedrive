use crate::CompressedCRDTOperations;

use sd_cloud_api::RequestConfigProvider;
use sd_core_sync::{GetOpsArgs, SyncMessage, NTP64};
use sd_prisma::prisma::{instance, PrismaClient};
use sd_utils::from_bytes_to_uuid;
use uuid::Uuid;

use std::{sync::Arc, time::Duration};

use tokio::time::sleep;

use super::err_break;

pub async fn run_actor(
	db: Arc<PrismaClient>,
	library_id: Uuid,
	sync: Arc<sd_core_sync::Manager>,
	cloud_api_config_provider: Arc<impl RequestConfigProvider>,
) {
	loop {
		loop {
			let instances = err_break!(
				db.instance()
					.find_many(vec![])
					.select(instance::select!({ pub_id }))
					.exec()
					.await
			)
			.into_iter()
			.map(|i| from_bytes_to_uuid(&i.pub_id))
			.collect::<Vec<_>>();

			let req_adds = err_break!(
				sd_cloud_api::library::message_collections::request_add(
					cloud_api_config_provider.cloud_api_config().await,
					library_id,
					instances,
				)
				.await
			);

			let mut instances = vec![];

			use sd_cloud_api::library::message_collections::do_add;

			for req_add in req_adds {
				let ops = err_break!(
					sync.get_ops(GetOpsArgs {
						count: 1000,
						clocks: vec![(
							req_add.instance_uuid,
							NTP64(
								req_add
									.from_time
									.unwrap_or_else(|| "0".to_string())
									.parse()
									.expect("couldn't parse ntp64 value"),
							),
						)],
					})
					.await
				);

				if ops.is_empty() {
					continue;
				}

				let start_time = ops[0].timestamp.0.to_string();
				let end_time = ops[ops.len() - 1].timestamp.0.to_string();

				instances.push(do_add::Input {
					uuid: req_add.instance_uuid,
					key: req_add.key,
					start_time,
					end_time,
					contents: serde_json::to_value(CompressedCRDTOperations::new(ops))
						.expect("CompressedCRDTOperation should serialize!"),
				})
			}

			if instances.is_empty() {
				break;
			}

			err_break!(
				do_add(
					cloud_api_config_provider.cloud_api_config().await,
					library_id,
					instances,
				)
				.await
			);
		}

		{
			// recreate subscription each time so that existing messages are dropped
			let mut rx = sync.subscribe();

			// wait until Created message comes in
			loop {
				if let Ok(SyncMessage::Created) = rx.recv().await {
					break;
				};
			}
		}

		sleep(Duration::from_millis(1000)).await;
	}
}
