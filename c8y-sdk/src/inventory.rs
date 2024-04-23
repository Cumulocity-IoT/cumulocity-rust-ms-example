use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::cumulocity_error::CumulocityError;

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ManagedObject {
    pub id: String,
    pub name: Option<String>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct CreateManagedObject {
    pub name: Option<String>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Clone)]
pub struct Inventory {
    pub base_url: String,
    pub tenant: String,
    pub username: String,
    pub password: String,
}

impl Inventory {
    pub async fn get_managed_object(&self, id: String) -> Result<ManagedObject, CumulocityError> {
        let client = Client::new();
        Ok(client
            .get(format!("{}/inventory/managedObjects/{id}", self.base_url))
            .basic_auth(
                format!("{}/{}", self.tenant, self.username),
                Some(self.password.clone()),
            )
            .header("accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .json::<ManagedObject>()
            .await?)
    }

    pub async fn delete_managed_object(&self, id: String) -> Result<String, CumulocityError> {
        let client = Client::new();
        Ok(client
            .delete(format!("{}/inventory/managedObjects/{id}", self.base_url))
            .basic_auth(
                format!("{}/{}", self.tenant, self.username),
                Some(self.password.clone()),
            )
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }

    pub async fn create_managed_object(
        &self,
        mo: CreateManagedObject,
    ) -> Result<ManagedObject, CumulocityError> {
        let client = Client::new();
        Ok(client
            .post(format!("{}/inventory/managedObjects", self.base_url))
            .basic_auth(
                format!("{}/{}", self.tenant, self.username),
                Some(self.password.clone()),
            )
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .json(&mo)
            .send()
            .await?
            .error_for_status()?
            .json::<ManagedObject>()
            .await?)
    }

    pub async fn update_managed_object(
        &self,
        mo: ManagedObject,
    ) -> Result<ManagedObject, CumulocityError> {
        let client = Client::new();
        let update = CreateManagedObject {
            name: mo.name,
            other: mo.other,
        };
        Ok(client
            .put(format!(
                "{}/inventory/managedObjects/{}",
                self.base_url, mo.id
            ))
            .basic_auth(
                format!("{}/{}", self.tenant, self.username),
                Some(self.password.clone()),
            )
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .json(&update)
            .send()
            .await?
            .error_for_status()?
            .json::<ManagedObject>()
            .await?)
    }
}
