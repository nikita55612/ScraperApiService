#![allow(warnings)]
use serde::{Deserialize, Serialize};
use super::super::utils::timestamp_now;
use super::error::*;

type Error = Box<dyn std::error::Error>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Product {
  pub mp: MP,
  pub id: String,
  pub url: String,
}

impl Product {
    pub fn from_string(s: &str) -> Result<Self, ScraperError> {
        let (mp, id) = s.split_once('/')
            .ok_or(ScraperError::ParseProduct)?;
        let mp = MP::from_str(mp)?;
        let id = match mp {
            MP::OZ | MP::WB => id.parse::<u64>()
                .map_err(|_| ScraperError::InvalidProductId)?
                .to_string(),
            MP::MM => {
                let _ = id.replacen('_', "", 1).parse::<u64>()
                    .map_err(|_| ScraperError::InvalidProductId)?;
                id.to_string()
            },
            _ => id.to_string(),
        };
        Ok (
            Self {
                url: symbol_to_url(&mp, &id),
                mp,
                id,
            }
        )
    }

    pub fn get_parse_url(&self) -> String {
        match self.mp {
            MP::OZ => format!("https://www.ozon.ru/api/entrypoint-api.bx/page/json/v2?url=/product/{}/", self.id),
            MP::WB => format!("https://card.wb.ru/cards/v2/detail?appType=1&curr=rub&dest=-1257218&nm={}", self.id),
            MP::PD => format!("https://www.podrygka.ru/catalog/{}-/", self.id),
            MP::MM => format!("https://megamarket.ru/promo-page/details/#?slug={}", self.id),
            _ => self.url.clone(),
        }
    }
}

fn symbol_to_url(mp: &MP, id: &str) -> String {
    match mp {
        MP::OZ => format!("https://www.ozon.ru/product/{}", id),
        MP::WB => format!("https://www.wildberries.ru/catalog/{}/detail.aspx", id),
        MP::PD => format!("https://www.podrygka.ru/catalog/{}-/", id),
        MP::MM => format!("https://megamarket.ru/promo-page/details/#?slug={}", id),
        _ => String::new(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MP {
    OZ,
    WB,
    YM,
    VI,
    MM,
    PD,
}

impl std::fmt::Display for MP {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl MP {
    pub fn from_str(s: &str) -> Result<MP, ScraperError> {
        match s.to_lowercase().as_str() {
            "oz" => Ok(MP::OZ),
            "wb" => Ok(MP::WB),
            "pd" => Ok(MP::PD),
            "mm" => Ok(MP::MM),
            _ => Err(ScraperError::InvalidMP),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct ProductData {
    name: Option<String>,
    price: Option<u64>,
    cprice: Option<u64>,
    seller: Option<String>,
    #[serde(rename = "sellerId", skip_serializing_if = "Option::is_none")]
    seller_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    img: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reviews: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    brand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductDataOutput {
    source: Product,
    data: Option<ProductData>,
    ts: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParseProductResult {
    Data(ProductData),
    Error(String)
}

impl ProductDataOutput {
    pub fn new(product: Product) -> Self {
        Self {
            source: product,
            data: None,
            ts: timestamp_now(),
        }
    }
}
