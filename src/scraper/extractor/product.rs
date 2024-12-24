use scraper::Html;
use std::fs;
use std::io;
use super::selectors;
use serde_json::Value;


#[derive(Clone, Debug, PartialEq, Default)]
pub struct ProductData {
    name: Option<String>,
    price: Option<u64>,
    cprice: Option<u64>,
    seller: Option<String>,
    seller_id: Option<String>,
    img: Option<String>,
    reviews: Option<u64>,
    rating: Option<f64>,
    brand: Option<String>,
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
}

fn read_file_to_string(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

fn oz_extractor(content: &str) -> Option<ProductData> {
	let html = Html::parse_document(content);
	let json = html
		.select(selectors::get("oz/data"))
		.next()
		.map(|s| s.inner_html())
		.and_then(|v| serde_json::from_str::<Value>(&v).ok())?;

	let widget_states = json.get("widgetStates")?;
	let tracking_info = json.get("layoutTrackingInfo");

	let mut pd = ProductData::default();

	for (key, value) in widget_states.as_object()? {
		if let Some(value_str) = value.as_str() {
			match key {
				k if k.starts_with("webPrice-") => {
					if let Ok(web_price) = serde_json::from_str::<Value>(value_str) {
						pd.price = web_price.get("price")
							.and_then(|v| v.as_str())
							.and_then(|s| s.replace(['\u{2009}', '₽'], "").parse().ok());
						pd.cprice = web_price.get("cardPrice")
							.and_then(|v| v.as_str())
							.and_then(|s| s.replace(['\u{2009}', '₽'], "").parse().ok());
					}
				},
				k if k.starts_with("webStickyProducts-") => {
					if let Ok(sticky_products) = serde_json::from_str::<Value>(value_str) {
						pd.name = sticky_products.get("name")
							.and_then(|v| v.as_str())
							.map(str::trim)
							.map(String::from);
						if let Some(seller) = sticky_products.get("seller") {
							pd.seller = seller.get("name")
							.and_then(|v| v.as_str())
							.map(str::trim)
							.map(String::from);
						pd.seller_id = seller.get("link")
							.and_then(|v| v.as_str())
							.and_then(|s| s.rsplit('/').nth(1))
							.map(String::from);
						}
					}
				},

				k if k.starts_with("webGallery-") => {
					if let Ok(gallery) = serde_json::from_str::<Value>(value_str) {
						pd.img = gallery.get("coverImage")
							.and_then(|v| v.as_str())
							.map(String::from);
					}
				},

				k if k.starts_with("webReviewProductScore") => {
					if let Ok(review_product_score) = serde_json::from_str::<Value>(value_str) {
						pd.reviews = review_product_score.get("reviewsCount").and_then(|v| v.as_u64());
						pd.rating = review_product_score.get("totalScore").and_then(|v| v.as_f64());
					}
				},
				_ => {}
			}
		}
	}

	if let Some(tracking_info) = tracking_info.and_then(|v| v.as_str()) {
		if let Ok(tracking_data) = serde_json::from_str::<Value>(tracking_info) {
			pd.brand = tracking_data.get("brandName").and_then(|v| v.as_str()).map(String::from);
		}
	}

	pd.to_option()
}

fn wb_extractor(content: &str) -> Option<ProductData> {
    let json  = serde_json::from_str::<Value>(content).ok()?;
	let data = json.get("data")
		.and_then(|v| v.get("products"))
		.and_then(|v| v.get(0))?;

	let mut pd = ProductData::default();

	pd.price = data.get("sizes")
		.and_then(|v| v.get(0))
		.and_then(|v| v.get("price"))
		.and_then(|v| v.get("total"))
		.and_then(|v| v.as_u64())
		.map(|v| (v as f64 / 100.0) as u64);
	if let Some(price) = pd.price {
		pd.cprice = Some((price as f64 * 0.98) as u64);
	}

	pd.name = data.get("name").and_then(|v| v.as_str()).map(String::from);
	pd.seller = data.get("supplier").and_then(|v| v.as_str()).map(String::from);
	pd.brand = data.get("brand").and_then(|v| v.as_str()).map(String::from);
	pd.reviews = data.get("feedbacks").and_then(|v| v.as_u64());
	pd.rating = data.get("reviewRating").and_then(|v| v.as_f64());
    pd.seller_id = data.get("supplierId").map(|v| v.to_string());

	pd.to_option()
}

fn mm_extractor(content: &str) -> Option<ProductData> {
	let html = Html::parse_document(content);
	let main = html.select(
		selectors::get("mm/main")
	).next()?;

	let mut pd = ProductData::default();

	pd.name = main.select(
		selectors::get("mm/product_title")
	).next()
		.map(|v| v.inner_html().trim().into());

	pd.price = main.select(
		selectors::get("mm/price_block")
	).next()
		.and_then(|v| v.attr("content"))
		.and_then(|v| v.parse::<u64>().ok());

	if let Some(price) = pd.price {
		pd.cprice = main.select(
			selectors::get("mm/bonus_amount")
		).next()
			.and_then(|v| v.inner_html()
				.replace(' ', "").trim().parse::<u64>().ok()
			).map(|v| price - v)
	}

	pd.seller = main.select(
		selectors::get("mm/seller")
	).next()
		.map(|v| v.inner_html()
			.replace(" (со склада МегаМаркет)", "")
			.into());

	pd.img = main
		.select(selectors::get("mm/img"))
		.next()
		.and_then(|v| v.attr("src"))
		.map(String::from);

	pd.rating = main
		.select(selectors::get("mm/rating"))
		.next()
		.map(|v| v.inner_html())
		.and_then(|v| v.parse::<f64>().ok());

	pd.reviews = main
		.select(selectors::get("mm/reviews"))
		.next()
		.map(|v| v.inner_html().trim().to_string())
		.and_then(|v| v.rsplitn(2, ' ').nth(1)
			.map(|s| s.replace(' ', "").to_string()))
		.and_then(|v| v.parse::<u64>().ok());


	pd.brand = main
		.select(selectors::get("mm/categories"))
		.last()
		.map(|v| v.inner_html());

	pd.to_option()
}

fn ym_extractor(content: &str) -> Option<ProductData> {
	let html = Html::parse_document(content);
	let card_content = html.select(
		selectors::get("ym/card_content")
	).next()?;

	let mut pd = ProductData::default();

	pd.name = card_content.select(
		selectors::get("ym/product_title")
	).next()
		.map(|v| v.inner_html());


	if let Some(price_data) = card_content
		.select(selectors::get("ym/price_data"))
		.next()
		.and_then(|v| v.attr("data-zone-data"))
		.and_then(|v| serde_json::from_str::<Value>(v).ok())
	{
		if let Some(price_details) = price_data.get("priceDetails") {
			pd.price = price_details.get("price")
				.and_then(|v| v.get("value"))
				.and_then(|v| v.as_u64());

			pd.cprice = price_details.get("greenPrice")
				.and_then(|v| v.get("price"))
				.and_then(|v| v.get("value"))
				.and_then(|v| v.as_u64());
		}
	}

	if let Some(shop_item) = card_content
		.select(selectors::get("ym/shop_item"))
		.next()
	{
		pd.seller = shop_item.select(
			selectors::get("span")
		).next()
			.map(|v| v.inner_html());


		pd.seller_id = shop_item.select(
			selectors::get("a")
		).next()
			.and_then(|v| v.attr("href"))
			.and_then(|v| v.rsplitn(2, '/').next())
			.map(String::from);
	}

	pd.img = card_content
		.select(selectors::get("ym/image_gallery"))
		.next()
		.and_then(|v| v.select(
			selectors::get("img")
		).next())
		.and_then(|v| v.attr("src"))
		.map(String::from);

	if let Some(rating_data) = card_content
		.select(selectors::get("ym/product_rating"))
		.next()
		.and_then(|v| v.select(
			selectors::get("noframes")
			).next()
		)
		.map(|v| v.inner_html())
		.and_then(|v| serde_json::from_str::<Value>(&v).ok())
	{
		if let Some(collections) = rating_data.get("collections") {
			pd.rating = collections.get("businessRatingStats")
				.and_then(|v| v.as_object())
				.and_then(|v| v.values().next())
				.and_then(|v| v.get("ratingValue"))
				.and_then(|v| v.as_f64())
				.map(|v| (v * 100.0).round() / 100.0);

			pd.reviews = collections.get("businessReviewStats")
				.and_then(|v| v.as_object())
				.and_then(|v| v.values().next())
				.and_then(|v| v.get("reviewsCount"))
				.and_then(|v| v.as_u64());
		}
	}

	pd.brand = card_content
		.select(selectors::get("ym/product_vendor"))
		.next()
		.and_then(|v| v.select(
			selectors::get("a")
		).next())
		.and_then(|v| v.select(
			selectors::get("span")
		).next())
		.map(|v| v.inner_html());

	pd.to_option()
}

#[cfg(test)]
mod tests {
	use super::*;

	// Добавить в конфигурацию возможность включения и отключения доступа к методу апи по токену доступа !!!!!!!!!


	#[test]
    fn test_ym_extractor() {
        let html_string = read_file_to_string("samples/ym/3.html").unwrap();
		let product_data = ym_extractor(&html_string).unwrap();

		println!("{:#?}", product_data);

        assert_eq!(true, true);
    }

	#[test]
    fn test_mm_extractor() {
        let html_string = read_file_to_string("samples/mm/1.html").unwrap();
		let product_data = mm_extractor(&html_string).unwrap();

		println!("{:#?}", product_data);

        assert_eq!(true, true);
    }

	#[test]
    fn test_wb_extractor() {
        let html_string = read_file_to_string("samples/wb/1.json").unwrap();
		let product_data = wb_extractor(&html_string).unwrap();

		println!("{:#?}", product_data);

        assert_eq!(true, true);
    }

	#[test]
    fn test_oz_extractor() {
        let html_string = read_file_to_string("samples/oz/2.html").unwrap();
		let product_data = oz_extractor(&html_string).unwrap();

		println!("{:#?}", product_data);

        assert_eq!(true, true);
    }
}
