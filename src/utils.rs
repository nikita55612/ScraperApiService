use chrono::Local as LocalTime;
use rand::{
    distributions::Alphanumeric,
    // seq::SliceRandom,
    thread_rng,
    Rng,
};
use sha1::{Digest, Sha1};
use std::{collections::HashSet, ffi::OsString, io::Write, net::TcpListener, path::Path};

use crate::config as cfg;

#[inline]
pub fn mkdir_if_not_exists<T: AsRef<Path>>(path: T) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

#[inline]
pub fn write_to_file<P: AsRef<Path>>(file_path: P, content: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)?;

    file.write_all(content)
}

pub fn print_logo() {
    println!("{}", cfg::LOGO)
}

pub fn is_port_open(port: u16) -> bool {
    TcpListener::bind((cfg::get().server.host.as_str(), port)).is_ok()
}

#[inline]
pub fn timestamp_now() -> u64 {
    LocalTime::now().timestamp() as u64
}

#[inline]
pub fn random_string(len: usize) -> String {
    let mut rng = thread_rng();
    (0..len).map(|_| rng.sample(Alphanumeric) as char).collect()
}

// pub fn create_uuid() -> String {
//     uuid::Uuid::new_v4().simple()
//         .to_string()
// }

#[inline]
pub fn create_token_id() -> String {
    format!("rs.{}", random_string(25))
}

#[inline]
pub fn sha1_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let res = hasher.finalize();
    hex::encode(res)
}

#[inline]
pub fn read_file<T: AsRef<Path>>(path: T) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

#[inline]
pub fn list_dir<T: AsRef<Path>>(dir: T) -> std::io::Result<Vec<OsString>> {
    Ok(std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|v| v.file_name())
        .collect::<Vec<_>>())
}

#[inline]
pub fn remove_all_dirs<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        }
    }
    Ok(())
}

#[inline]
pub fn remove_duplicates<T>(l: &mut Vec<T>)
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut seen = HashSet::new();
    l.retain(|i| seen.insert(i.clone()));
}

// pub fn select_random_product_name() -> &'static str {
//     const products: [&str; 40] = [
//         "Майнинг ферма домашних животных",
//         "Солнцезащитный зонт с LED-подсветкой",
//         "Эко-рюкзак с солнечной панелью",
//         "Умные носки с GPS-трекером",
//         "Карманный голографический проектор",
//         "Робот-пылесос с искусственным интеллектом",
//         "Многофункциональная кухонная перчатка",
//         "Портативный очиститель воздуха",
//         "Умная расческа с анализатором волос",
//         "Bluetooth-наклейки для поиска вещей",
//         "Водонепроницаемый планшет для душа",
//         "Термос с встроенным измельчителем зубов",
//         "Экологичный ECO+ органайзер для завтрака",
//         "Умная бутылка с напоминаниями о воде",
//         "Мини-телепорт для домашних растений",
//         "Дрон-фотограф для селфи с зумом",
//         "Умные наушники с переводчиком в комплекте",
//         "Многоразовая электронная записная книжка 60 листов",
//         "Портативный генератор снега",
//         "Самоочищающаяся миска для домашних животных",
//         "Складная электрическая доска для серфинга",
//         "Aromatherapy-часы с эфирными маслами",
//         "Интерактивный коврик для медитации",
//         "Умная зубная щетка с AI-анализом",
//         "Персональный метеорологический датчик",
//         "Мягкая робо-подушка с массажем и развитием",
//         "Очки с поддополненной реальностью",
//         "Электронный питомец-голограмма",
//         "Универсальный адаптер для зарядки мыслей",
//         "Умный ошейник для домашних животных",
//         "Нейро-массажёр для мозга",
//         "Карманный генератор анти радуги",
//         "Грибной чай с эффектом понимания",
//         "Коллекционные флаконы с ароматами цивилизаций",
//         "Космический гармонизатор ауры с функцией цветопередачи",
//         "Музыкальная зубочистка с эквалайзером",
//         "Умная вешалка с климат-контролем",
//         "Самонагревающийся суп-конструктор с ложкой",
//         "Пылесос ЧистоШторм согревающий душу",
//         "Крем Звёздный шёпот для бровей и рук"
//     ];

//     products.choose(&mut thread_rng()).unwrap()
// }

// pub fn select_random_vendor() -> &'static str {
//     const vendors: [&str; 30] = [
//         "ТехноТрейд Маркет",
//         "ГлобалШоп Инновации",
//         "СмартСейл Экспресс",
//         "АзияТех Импорт",
//         "ЭкоСтиль Торг",
//         "МегаБренд Сервис",
//         "Ретейл Прогресс",
//         "НеваТрейд Групп",
//         "АльфаМаркет Решения",
//         "СибирьКомерц",
//         "ВысотаТорг",
//         "ПремиумПродукт Альянс",
//         "КонтинентТрейд",
//         "УниверсалМаркет",
//         "СтандартТех Импорт",
//         "РегионСнаб",
//         "КачествоСервис",
//         "ИнтерТрейд Логистик",
//         "НоваяВолна Маркет",
//         "АвангардШоп",
//         "МирТоваров Экспресс",
//         "СтратегияТорг",
//         "РесурсМаркет",
//         "ТочкаРоста Трейд",
//         "АтлантТех",
//         "ИнновационныйСоюз",
//         "КомфортТрейд",
//         "МобильныйМир",
//         "ГарантПродукт",
//         "РазвитиеТорг"
//     ];

//     vendors.choose(&mut thread_rng()).unwrap()
// }

// pub fn select_random_brand() -> &'static str {
//     const brands: [&str; 30] = [
//         "Abibas",
//         "КалвинКлайнер",
//         "ЛуиВитончик",
//         "ЮникЛоудинг",
//         "ТесляМоторс",
//         "Asas",
//         "ЛейзСкрипучий",
//         "Кока-НеКола",
//         "Фэйкбук",
//         "ТехноСвет",
//         "АкваЛайн",
//         "Gogle",
//         "СнэкПро",
//         "SpectraVolt",
//         "FlowLuxe",
//         "Quantumex",
//         "Zenithon",
//         "Open&AI",
//         "RapidForce",
//         "Мегафорс",
//         "Вижионет",
//         "AppleLuxe",
//         "Линкорн",
//         "Самсон",
//         "Спектролайн",
//         "СилкПро",
//         "СтарГлайд",
//         "Альтроника",
//         "Энерджайз",
//         "DiamondElysium"
//     ];

//     brands.choose(&mut thread_rng()).unwrap()
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_create_token_id() {
        for _ in 0..100 {
            println!("{}", create_token_id());
        }
        assert_eq!(true, true);
    }

    #[test]
    fn test_mkdir() {
        let path = Path::new(&cfg::get().browser.users_temp_data_dir);
        mkdir_if_not_exists(path).unwrap();
        assert_eq!(true, true);
    }

    #[test]
    fn test_is_port_open() {
        assert_eq!(true, is_port_open(51081));
    }

    #[test]
    fn test_remove_all_directories() {
        remove_all_dirs("./users_temp_data").unwrap();
        assert_eq!(true, true);
    }

    #[test]
    fn test_remove_list_dub() {
        let mut list = vec![
            "123", "123", "123", "123", "123", "123", "1235", "1234", "1233", "1232", "1231",
        ];
        println!("{:?}", list);
        let mut seen = HashSet::new();
        list.retain(|item| seen.insert(item.to_string()));
        println!("{:?}", list);
        assert_eq!(true, true);
    }
}
