use azure_sdk_core::prelude::*;
use azure_sdk_cosmos::prelude::*;
use futures::stream::StreamExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // First we retrieve the account name and master key from environment variables.
    // We expect master keys (ie, not resource constrained)
    let master_key =
        std::env::var("COSMOS_MASTER_KEY").expect("Set env variable COSMOS_MASTER_KEY first!");
    let account = std::env::var("COSMOS_ACCOUNT").expect("Set env variable COSMOS_ACCOUNT first!");

    // This is how you construct an authorization token.
    // Remember to pick the correct token type.
    // Here we assume master.
    // Most methods return a ```Result<_, AzureError>```.
    // ```AzureError``` is an enum union of all the possible underlying
    // errors, plus Azure specific ones. For example if a REST call returns the
    // unexpected result (ie NotFound instead of Ok) we return an Err telling
    // you that.
    let authorization_token = AuthorizationToken::new_master(&master_key)?;

    // Once we have an authorization token you can create a client instance. You can change the
    // authorization token at later time if you need, for example, to escalate the privileges for a
    // single operation.
    let client = ClientBuilder::new(&account, authorization_token)?;

    // The Cosmos' client exposes a lot of methods. This one lists the databases in the specified
    // account. Database do not implement Display but deref to &str so you can pass it to methods
    // both as struct or id.
    let databases = client.list_databases().execute().await?;

    println!(
        "Account {} has {} database(s)",
        account,
        databases.databases.len()
    );

    // Each Cosmos' database contains one or more collections. We can enumerate them using the
    // list_collection method.
    for db in databases.databases {
        let database_client = client.with_database_client(&db.id);
        let stream = Box::pin(
            database_client
                .list_collections()
                .with_max_item_count(2)
                .with_activity_id("prova"),
        );
        let mut stream = Box::pin(stream.stream());
        while let Some(res) = stream.next().await {
            let res = res?;
            println!(
                "database {} has {} collection(s)",
                db.id,
                res.collections.len()
            );
        }
    }
    Ok(())
}
