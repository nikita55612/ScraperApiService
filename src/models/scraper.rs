#![allow(warnings)]
use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize
};
use rand::Rng;

use super::super::{
    config as cfg,
    scraper::error::ScraperError,
    utils::{
        select_random_product_name,
        select_random_vendor,
        select_random_brand,
        timestamp_now
    }
};


pub static AVAILABLE_MARKETS: Lazy<Vec<String>> = Lazy::new(||
    cfg::get().api.available_markets.clone()
);

pub static MARKET_MAP: Lazy<HashMap<String, Market>> = Lazy::new(|| {
    HashMap::from([
        (
            "oz".into(),
            Market {
                name: "Ozon".into(),
                url: "https://ozon.ru".into(),
                available: AVAILABLE_MARKETS.contains(&"oz".into())
            }
        ),
        (
            "wb".into(),
            Market {
                name: "Wildberries".into(),
                url: "https://www.wildberries.ru/".into(),
                available: AVAILABLE_MARKETS.contains(&"wb".into())
            }
        ),
        (
            "ym".into(),
            Market {
                name: "YandexMarket".into(),
                url: "https://market.yandex.ru/".into(),
                available: AVAILABLE_MARKETS.contains(&"ym".into())
            }
        ),
        (
            "mm".into(),
            Market {
                name: "MegaMarket".into(),
                url: "https://megamarket.ru/".into(),
                available: AVAILABLE_MARKETS.contains(&"mm".into())
            }
        ),
    ])
});

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct ProductData {
    pub sku: String,

	#[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    pub url: String,

	#[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<u64>,

	#[serde(skip_serializing_if = "Option::is_none")]
    pub cprice: Option<u64>,

	#[serde(skip_serializing_if = "Option::is_none")]
    pub seller: Option<String>,

    #[serde(rename = "sellerId", skip_serializing_if = "Option::is_none")]
    pub seller_id: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
    pub img: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
    pub reviews: Option<u64>,

	#[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f64>,

	#[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>
}

impl ProductData {
	pub fn is_empty(&self) -> bool {
        [
            self.name.is_none(),
            self.price.is_none(),
            self.cprice.is_none(),
            self.seller.is_none(),
            self.seller_id.is_none(),
            self.img.is_none(),
            self.reviews.is_none(),
            self.rating.is_none(),
            self.brand.is_none(),
        ]
        .iter()
        .all(|is_none| *is_none)
	}

	pub fn to_option(self) -> Option<ProductData> {
		if self.is_empty() {
			return None;
		}

		Some(self)
	}

	pub fn rand() -> Self {
        let mut rng = rand::thread_rng();

        let price = rng.gen_range(200..9000) as u64;
        let card_price = (price as f64 * rng.gen_range(0.5..0.95)) as u64;

        Self {
            name: Some(
				select_random_product_name().into()
			),
            price: Some(price),
            cprice: Some(card_price),
            seller: Some(
				select_random_vendor().into()
			),
            reviews: Some(
				rng.gen_range(0..4500) as u64
			),
            rating: Some(
				rng.gen_range(0.0..5.0)
			),
			seller_id: Some(
				rng.gen_range(10000000..999999999).to_string()
			),
			brand: Some(
				select_random_brand().into()
			),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="lowercase")]
pub enum Symbol {
    OZ,
    WB,
    YM,
    MM
}

impl Symbol {
    pub fn from_string(s: &str) -> Result<Self, ScraperError> {
        match s.to_lowercase().as_str() {
            "oz" => Ok(Self::OZ),
            "wb" => Ok(Self::WB),
            "ym" => Ok(Self::YM),
            "mm" => Ok(Self::MM),
            _ => Err(ScraperError::InvalidSymbol)
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OZ => "oz",
            Self::WB => "wb",
            Self::YM => "ym",
            Self::MM => "mm"
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Market {
    pub name: String,
    pub url: String,
    pub available: bool
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Product {
    pub symbol: Symbol,
    pub id: String,
    pub sku: String,
    pub url: String
}

impl Product {
    pub fn from_string_without_valid(s: &str) -> Self {
        let (symbol, id) = s.split_once('/').unwrap();
        let symbol = Symbol::from_string(symbol).unwrap();

        let url = match symbol {
            Symbol::OZ => format!("https://www.ozon.ru/product/{}", id),
            Symbol::WB => format!("https://www.wildberries.ru/catalog/{}/detail.aspx", id),
            Symbol::YM => {
                let parts = id.splitn(3, '-').collect::<Vec<_>>();
                format!(
                    "https://market.yandex.ru/product/{}?sku={}&uniqueId={}",
                    parts[0], parts[1], parts[2],
                )
            },
            Symbol::MM => format!("https://megamarket.ru/promo-page/details/#?slug={}", id),
        };
        let sku = match symbol {
            Symbol::YM => id.splitn(3, '-')
                .nth(1)
                .unwrap()
                .into(),
            _ => id.to_string()
        };

        Self {
            id: id.into(),
            symbol,
            sku,
            url
        }
    }

    pub fn get_parse_url(&self) -> String {
        match self.symbol {
            Symbol::OZ => format!("https://www.ozon.ru/api/entrypoint-api.bx/page/json/v2?url=/product/{}/", self.id),
            Symbol::WB => format!("https://card.wb.ru/cards/v2/detail?appType=1&curr=rub&dest=-1257218&nm={}", self.id),
            Symbol::YM => {
                let parts = self.id.splitn(3, '-').collect::<Vec<_>>();
                format!(
                    "https://market.yandex.ru/product/{}?sku={}&uniqueId={}",
                    parts[0], parts[1], parts[2],
                )
            },
            Symbol::MM => format!("https://megamarket.ru/promo-page/details/#?slug={}", self.id),
            _ => self.url.clone(),
        }
    }
}
