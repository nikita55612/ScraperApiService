use async_stream::stream;
use tokio_stream::Stream;
use super::models::{
    Task, 
    TaskProgress, 
    TaskStatus,
    TaskResult,
    ProductResult
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::thread_rng;


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


impl ProductData {
    fn rand() -> Self {
        let mut rng = rand::thread_rng();
        let price = rng.gen_range(140..1200) as u64;
        let card_price = (price as f64 * rng.gen_range(0.5..0.95)) as u64;
        Self {
            name: Some(select_random_product().into()),
            price: Some(price),
            cprice: Some(card_price),
            seller: Some(select_random_marketplace_vendor().into()),
            reviews: Some(rng.gen_range(0..2500) as u64),
            rating: Some(rng.gen_range(0.0..5.0)),
            ..Default::default()
        }
    }
}


pub fn task_stream(mut task: Task) -> impl Stream<Item = Task> {

    let s = stream! {

        task.init_progress();
        task.set_status(TaskStatus::Processing);
        task.init_result_data();

        while !matches!(task.status, TaskStatus::Completed | TaskStatus::Error) {

            tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;

            let order_item = task.order.items.get(
                task.get_curr_step() as usize
            ).unwrap().clone();

            let rand_product_data = ProductData::rand();
            let product_result = ProductResult::Data(rand_product_data);

            task.insert_result_item(order_item, product_result);

            task.next_progress_step();
            if task.is_done() {
                task.set_status(TaskStatus::Completed);
            }
            yield task.clone();
        }
    };
    Box::pin(s)
}


fn select_random_product() -> &'static str {
    let products = [
        "Электронная чашка-хамелеон",
        "Солнцезащитный зонт с LED-подсветкой",
        "Эко-рюкзак с солнечной панелью",
        "Умные носки с GPS-трекером",
        "Карманный голографический проектор",
        "Робот-пылесос с искусственным интеллектом",
        "Многофункциональная кухонная перчатка",
        "Портативный очиститель воздуха", 
        "Умная расческа с анализатором волос",
        "Bluetooth-наклейки для поиска вещей",
        "Водонепроницаемый планшет для душа",
        "Термос с встроенным измельчителем",
        "Экологичный органайзер для завтрака",
        "Умная бутылка с напоминаниями о воде",
        "Мини-телепорт для домашних растений",
        "Дрон-фотограф для селфи",
        "Умные наушники с переводчиком",
        "Многоразовая электронная записная книжка",
        "Портативный генератор снега",
        "Самоочищающаяся миска для домашних животных",
        "Складная электрическая доска для серфинга",
        "Aromatherapy-часы с эфирными маслами",
        "Интерактивный коврик для медитации",
        "Умная зубная щетка с AI-анализом",
        "Персональный метеорологический датчик",
        "Мягкая робо-подушка с массажем",
        "Очки с дополненной реальностью",
        "Электронный питомец-голограмма",
        "Универсальный адаптер для зарядки мыслей",
        "Умный ошейник для домашних животных"
    ];

    products.choose(&mut thread_rng()).unwrap()
}

fn select_random_marketplace_vendor() -> &'static str {
    let vendors = [
        "ТехноТрейд Маркет",
        "ГлобалШоп Инновации",
        "СмартСейл Экспресс",
        "АзияТех Импорт",
        "ЭкоСтиль Торг",
        "МегаБренд Сервис",
        "Ретейл Прогресс",
        "НеваТрейд Групп",
        "АльфаМаркет Решения",
        "СибирьКомерц",
        "ВысотаТорг",
        "ПремиумПродукт Альянс",
        "КонтинентТрейд",
        "УниверсалМаркет",
        "СтандартТех Импорт",
        "РегионСнаб",
        "КачествоСервис",
        "ИнтерТрейд Логистик",
        "НоваяВолна Маркет",
        "АвангардШоп",
        "МирТоваров Экспресс",
        "СтратегияТорг",
        "РесурсМаркет",
        "ТочкаРоста Трейд",
        "АтлантТех",
        "ИнновационныйСоюз",
        "КомфортТрейд",
        "МобильныйМир",
        "ГарантПродукт",
        "РазвитиеТорг"
    ];

    vendors.choose(&mut thread_rng()).unwrap()
}

