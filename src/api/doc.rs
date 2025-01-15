use once_cell::sync::Lazy;
use serde::Serialize;
use utoipa::{
	openapi::{
		self,
		security::{
			HttpAuthScheme,
			HttpBuilder,
			SecurityScheme
		}
	},
	Modify,
	OpenApi
};


use super::{
    super::{
        api::app::ROOT_API_PATH,
        config::{
            self as cfg,
            Config,
        },
        models::{
            api::{
                ApiState,
                Order,
                Task,
                Token
            },
            scraper::Market
        }
    },
    error::ApiError
};


#[derive(Debug, Serialize)]
struct ApiToken;

impl Modify for ApiToken {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        if let Some(schema) = openapi.components.as_mut() {
            schema.add_security_scheme(
                "Token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
						.description(Some("Token предоставляет доступ к приватным api методам.\nПример токена: rs.qWzZgfMjXUhrwgZWn4uZRT9VK"))
                        .build(),
                ),
            );
        }
    }
}

pub static API_DESCRIPTION: Lazy<String> = Lazy::new(|| {
    if let Some(path) = &cfg::get().api.description_file_path {
        match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => DEFAULT_API_DESCRIPTION.into()
        }
    } else {
        DEFAULT_API_DESCRIPTION.into()
    }
});

const DEFAULT_API_DESCRIPTION: &'static str = r#"
Дата публикации: **1/16/25**

# Документация API

[Github page](https://github.com/Nikita55612/RustScraperApi)

---

## О проекте

RustScraper API - это мощный инструмент для парсинга данных о товарах с популярных маркетплейсов. API разработано на языке [Rust](https://ru.wikipedia.org/wiki/Rust_(%D1%8F%D0%B7%D1%8B%D0%BA_%D0%BF%D1%80%D0%BE%D0%B3%D1%80%D0%B0%D0%BC%D0%BC%D0%B8%D1%80%D0%BE%D0%B2%D0%B0%D0%BD%D0%B8%D1%8F)), что обеспечивает высокую производительность и надежность работы.

Проект разрабатывается и поддерживается одним человеком.

Мой контакт - [@Nikita5612](https://t.me/Nikita5612)

Доступ к сервису выдается на платной основе, подробности в лс.

---

## Основные возможности

- Поддержка крупнейших маркетплейсов:
  - [Wildberries](https://www.wildberries.ru/)
  - [Ozon](https://www.ozon.ru/)
  - [Яндекс.Маркет](https://market.yandex.ru/)
  - [МегаМаркет](https://megamarket.ru/)
- Гибкая система обхода блокировок через прокси-серверы
- Поддержка пользовательских cookies для сохранения настроек сессии
- [WebSocket](https://ru.wikipedia.org/wiki/WebSocket) подключение для отслеживания статуса парсинга в реальном времени
- Простой и понятный REST API интерфейс
- Детальная валидация входящих данных
- Система очередей для распределения нагрузки

---

## Начало работы

### 1. Получение тестового токена

Для начала работы с API необходимо получить тестовый токен через метод `/test-token`. Тестовый токен предоставляется автоматически для уникальных IP-адресов и имеет ограниченный срок действия.

### 2. Структура заказа

Заказ на парсинг состоит из трех основных компонентов:
- Список товаров (`products`)
- Пул прокси-серверов (`proxyPool`)
- Пользовательские cookies (`cookies`)

#### Форматы ссылок на товары
Поддерживается два формата указания товаров:
1. Короткий формат: `маркет/id`
   - `wb/145700662` ([Wildberries](https://www.wildberries.ru/))
   - `oz/1736756863` ([Ozon](https://www.ozon.ru/))
   - `ym/1732949807-100352880819` ([Яндекс.Маркет](https://market.yandex.ru/))
   - `mm/100065768905` ([МегаМаркет](https://megamarket.ru/))
2. Полный URL товара с маркетплейса

---

### 3. Отправка заказа и получение результатов

Процесс парсинга состоит из следующих шагов:
1. Валидация заказа через метод `/valid-order`
2. Отправка заказа методом `/order`
3. Получение `order_hash` для отслеживания статуса
4. Мониторинг выполнения через REST API или [WebSocket](https://ru.wikipedia.org/wiki/WebSocket)

---

## Особенности работы

### Система защиты от блокировок

API предоставляет два механизма защиты от блокировок:

1. **Прокси-пул (ProxyPool)**:
   - Позволяет распределять запросы через разные IP-адреса
   - Поддерживает формат: `USERNAME:PASSWORD@HOST:PORT`
   - Возможность указания нескольких прокси-серверов

2. **Пользовательские Cookies**:
   - Сохранение настроек авторизации
   - Поддержка геолокационных настроек
   - Сохранение пунктов выдачи заказов
   - Персонализированные настройки отображения цен

### Ограничения и лимиты

Каждый токен имеет следующие ограничения:
- Лимит на количество товаров в заказе
- Лимит на количество одновременных обработок
- Ограничение времени жизни токена (TTL)
- Лимит на количество [WebSocket](https://ru.wikipedia.org/wiki/WebSocket) подключений

---

## Мониторинг выполнения

### REST API мониторинг
Получение статуса выполнения через периодические запросы к методу `/task/{order_hash}`

### WebSocket мониторинг (рекомендуется)
Установка постоянного соединения через `/task-ws/{order_hash}` для получения обновлений в реальном времени

---

## Обработка ошибок

API использует унифицированную систему кодов ошибок. Каждая ошибка содержит:
- Текстовое описание (`error`)
- Числовой код (`code`)
- Информативное сообщение (`message`)

Подробное описание всех возможных ошибок представлено в таблице ApiError.

---

## ApiError

| Название | Описание | Код | HTTP код |
|----------------|-----------|-----|----------|
| **UnknownError** | Неизвестная ошибка сервера | **0** | 500 |
| **MissingAuthorizationHeader** | Отсутствует заголовок авторизации | **102** | 400 |
| **MalformedAuthorizationHeader** | Неверный формат заголовка авторизации. Ожидается формат 'Bearer <token>' | **103** | 401 |
| **InvalidAccessToken** | Предоставлен недействительный токен доступа | **104** | 401 |
| **AccessTokenExpired** | Срок действия токена доступа истек | **105** | 401 |
| **MissingUrlQueryParameter** | Отсутствует обязательный URL-параметр запроса | **200** | 400 |
| **InvalidUrlQueryParameter** | Недопустимое значение URL-параметра запроса | **201** | 400 |
| **InvalidOrderParameter** | Недопустимое значение параметра заказа | **202** | 400 |
| **InvalidOrderFormat** | Не удалось десериализовать тело запроса в объект заказа | **203** | 400 |
| **EmptyRequestBody** | Тело запроса пусто. Ожидается определенная структура | **204** | 400 |
| **EmptyOrder** | Отправленный заказ пуст | **205** | 400 |
| **QueueOverflow** | Очередь обработчика заполнена. Достигнуто максимальное количество задач | **300** | 409 |
| **ProductLimitExceeded** | Заказ превышает максимальный лимит продуктов | **301** | 409 |
| **ConcurrencyLimitExceeded** | Токен превысил лимит одновременной обработки | **302** | 409 |
| **DuplicateTask** | Задача с указанным order_hash уже существует | **303** | 409 |
| **WebSocketLimitExceeded** | Невозможно установить новое WebSocket-соединение, </br>так как сервер достиг максимального лимита одновременных подключений | **304** | 409 |
| **AccessRestricted** | Доступ к методу ограничен | **305** | 409 |
| **TokenDoesNotExist** | Токен не существует | **400** | 404 |
| **TaskNotFound** | Задача с указанным order_hash не существует | **401** | 404 |
| **PathNotFound** | Запрошенный путь не найден | **404** | 404 |
| **TaskSendFailure** | Не удалось отправить задачу обработчику | **500** | 500 |
| **ReqwestSessionError** | Ошибка сессии запроса | **501** | 500 |
| **DatabaseError** | Сбой транзакции базы данных | **502** | 500 |
| **SerializationError** | Не удалось сериализовать объект | **503** | 500 |
</br>
"#;

//////////////////////////////////
// API Documentation Structure  //
////////////////////////////////
#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "https://rustscraper.ru", description = "Remote API https"),
        (url = "http://rustscraper.ru", description = "Remote API http"),
        (url = "http://localhost:5050", description = "Local server for testing"),
    ),
    info(
        title = "RustScraperApi",
        description = &*API_DESCRIPTION,
        version = "1.0.0",
        contact(name = "Nikita", url = "https://t.me/Nikita5612")
    ),
    tags(
        (name = "order", description = "Методы отправки заказа на парсинг и получения статуса его выполнения"),
        (name = "token", description = "Методы получения информации о токене доступа и создания тестового токена"),
        (name = "utilities", description = "Утилиты для получения API информации")
    ),
    modifiers(&ApiToken),
    paths(
        myip,
        token_info,
        token_info_,
        test_token,
        openapi,
        config,
        state,
        markets,
        order,
        valid_order,
        task,
        task_ws
    ),
)]
pub struct ApiDoc;

////////////////////////////
// Utility Endpoints     //
//////////////////////////

#[utoipa::path(
    get,
    path = "/openapi.json",
    tags = ["utilities"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### GET /openapi.json
Метод для получения openapi.json - это файл, содержащий спецификацию API, написанную в формате OpenAPI. Этот файл описывает, как взаимодействовать с API, включая его конечные точки (endpoints), параметры, схемы данных, методы запросов, ответы и другие детали.

Для работы с API спецификациями в формате OpenAPI используйте [Swagger Editor](https://editor.swagger.io/).

*Инструменты для работы с openapi.json:*
- Swagger UI: визуализирует спецификацию в виде интерактивной документации.
- Postman: импортирует файл для упрощенного тестирования API.
- OpenAPI Generator: генерирует клиентские или серверные библиотеки на основе спецификации.
"#,
    responses(
        (status = 200, description = "openapi.json", content_type = "application/json")
    )
)]
#[allow(dead_code)]
fn openapi() {}

#[utoipa::path(
    get,
    path = "/ping",
    tags = ["utilities"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### GET /ping
Метод проверки работоспособности сервера. Используется для мониторинга состояния сервера и проверки его доступности.
При успешном выполнении возвращает строку "pong".
"#,
    responses(
        (status = 200, description = "pong", content_type = "text/plain")
    )
)]
#[allow(dead_code)]
fn ping() {}

#[utoipa::path(
    get,
    path = "/myip",
    tags = ["utilities"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### GET /myip
Метод для определения IP-адреса клиента. Возвращает текущий IP-адрес в формате SocketAddr (IP:PORT),
с которого производится обращение к API.
"#,
    responses(
        (status = 200, description = "SocketAddr", content_type = "text/plain")
    )
)]
#[allow(dead_code)]
fn myip() {}

////////////////////////////
// Token Management     //
//////////////////////////

#[utoipa::path(
    get,
    path = "/token-info",
    tags = ["token"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### GET /token-info

Метод для получения информации о текущем токене авторизации. Требует наличия валидного токена в заголовке запроса.

**Заголовок запроса:**
- Authorization: Bearer <YOUR-TOKEN>

**Параметры токена:**
- **id** - Строка токена для передачи в заголовок запроса
- **createdAt** - Дата и время создания токена в формате timestamp
- **ttl** - Время жизни токена в секундах
- **orderProductsLimit** - Лимит токена на количество товаров в заказе
- **taskCountLimit** - Лимит токена на количество параллельных обработок заказа

```python
import requests

headers = {
    "Authorization": "Bearer your-token-here"
}

response = requests.get("http://domain/api/v1/token-info", headers=headers)
print(response.json())
```
"#,
    security(
        ("Token" = [])
    ),
    responses(
        (
            status = 200, description = "Параметры токена", body = Token, content_type = "application/json",
            example = json!(
                {"id":"rs.voHvMvpmoFgakFbd7U2VMyTYh","createdAt":1736866893,"ttl":86400,"orderProductsLimit":40,"taskCountLimit":1}
            )
        ),
        (status = 400, description = r#"
### Ошибка ApiError

Значения ошибок смотреть в таблице ApiError
"#,
        body = ApiError, content_type = "application/json",
        example = json!({"error":"Unknown","code":0,"message":"Unknown server error."}))
    )
)]
#[allow(dead_code)]
fn token_info() {}

#[utoipa::path(
    get,
    path = "/token-info/{token_id}",
    tags = ["token"],
    context_path = &*&*ROOT_API_PATH,
    description = r#"
### GET /token-info/{token_id}

Метод для получения информации о конкретном токене по его идентификатору.
Позволяет получить детальную информацию о любом токене в системе.

Тот же функционал, что и */token-info*, только токен передается в пути запроса */{token_id}*

**Параметры токена:**
- **id** - Строка токена для передачи в заголовок запроса
- **createdAt** - Дата и время создания токена в формате timestamp
- **ttl** - Время жизни токена в секундах
- **orderProductsLimit** - Лимит токена на количество товаров в заказе
- **taskCountLimit** - Лимит токена на количество параллельных обработок заказа

**Пример:**
- /token-info/rs.qWzZgfMjXUhrwgZWn4uZRT9VK

```python
import requests

token_id = "rs.qWzZgfMjXUhrwgZWn4uZRT9VK"
response = requests.get(f"http://domain/api/v1/token-info/{token_id}")
print(response.json())
```
"#,
    params(
        ("token_id" = String, Path, description = "Параметр id токена"),
    ),
    responses(
        (
            status = 200, description = "Параметры токена", body = Token, content_type = "application/json",
            example = json!(
                {"id":"rs.voHvMvpmoFgakFbd7U2VMyTYh","createdAt":1736866893,"ttl":86400,"orderProductsLimit":40,"taskCountLimit":1}
            )
        ),
        (status = 400, description = r#"
### Ошибка ApiError

Значения ошибок смотреть в таблице ApiError
"#,
        body = ApiError, content_type = "application/json",
        example = json!({"error":"Unknown","code":0,"message":"Unknown server error."}))
    )
)]
#[allow(dead_code)]
fn token_info_() {}

#[utoipa::path(
    get,
    path = "/test-token",
    tags = ["token"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### GET /test-token
Метод для получения временного тестового токена доступа к API.
Позволяет получить ограниченный по времени токен для тестирования функциональности API.

**Ограничения:**
- Доступно только для уникальных IP-адресов
- Токен имеет ограниченный срок действия
- Может иметь ограниченный функционал

**Параметры токена:**
- **id** - Строка токена для передачи в заголовок запроса
- **createdAt** - Дата и время создания токена в формате timestamp
- **ttl** - Время жизни токена в секундах
- **orderProductsLimit** - Лимит токена на количество товаров в заказе
- **taskCountLimit** - Лимит токена на количество параллельных обработок заказа

```python
import requests

response = requests.get("http://domain/api/v1/test-token")
token_data = response.json()
print(token_data)  # Информация о тестовом токене
```
"#,
    responses(
        (
            status = 200, description = "Параметры тестового токена", body = Token, content_type = "application/json",
            example = json!(
                {"id":"rs.voHvMvpmoFgakFbd7U2VMyTYh","createdAt":1736866893,"ttl":86400,"orderProductsLimit":40,"taskCountLimit":1}
            )
        ),
        (status = 400, description = r#"
### Ошибка ApiError

Значения ошибок смотреть в таблице ApiError
"#,
        body = ApiError, content_type = "application/json",
        example = json!({"error":"Unknown","code":0,"message":"Unknown server error."}))
    )
)]
#[allow(dead_code)]
fn test_token() {}

////////////////////////////
// System Configuration  //
//////////////////////////

#[utoipa::path(
    get,
    path = "/config",
    tags = ["utilities"],
    context_path = &*ROOT_API_PATH,
    description = r##"
### GET /config
Метод получения текущей конфигурации API. Возвращает актуальные настройки и параметры работы API-сервера.
"##,
    responses(
        (status = 200, description = "Конфигурация API", body = Config, content_type = "application/json")
    )
)]
#[allow(dead_code)]
fn config() {}

#[utoipa::path(
    get,
    path = "/state",
    tags = ["utilities"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### GET /state
Метод получения текущего состояния API-сервера. Возвращает информацию о:
- Количестве активных обработчиков
- Лимите очереди задач (суммарный лимит для всех обработчиков)
- Текущем количестве задач в очереди
- Лимите открытых WebSocket соединений
- Текущем количестве открытых WebSocket соединений
"#,
    responses(
        (status = 200, description = "Состояние API", body = ApiState, content_type = "application/json")
    )
)]
#[allow(dead_code)]
fn state() {}

////////////////////////////
// Marketplace Methods   //
//////////////////////////

#[utoipa::path(
    get,
    path = "/markets",
    tags = ["utilities"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### GET /markets
Метод получения информации о доступных маркетплейсах в системе.
Возвращает список поддерживаемых маркетплейсов и их параметров в формате JSON.
"#,
    responses(
        (status = 200, description = "Доступные маркетплейсы", body = HashMap<String, Market>, content_type = "application/json",
        example = json!(
            {
                "wb": {"name": "Wildberries", "url": "https://www.wildberries.ru/", "available": true},
                "mm": {"name": "MegaMarket", "url": "https://megamarket.ru/", "available": true},
                "oz": {"name": "Ozon", "url": "https://ozon.ru", "available": true},
                "ym": {"name": "YandexMarket", "url": "https://market.yandex.ru/", "available": true}
            }
        )
    )
    )
)]
#[allow(dead_code)]
fn markets() {}

////////////////////////////
// Order Management      //
//////////////////////////

#[utoipa::path(
    post,
    path = "/order",
    tags = ["order"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### POST /order
Метод создания нового заказа в системе.

**Описание:**
Позволяет отправить заказ на обработку с указанием списка товаров и настроек прокси-серверов.

**Формат данных:**
- В параметре products можно указывать как прямые ссылки на товары, так и короткий формат "маркет/товар"
- Поддерживаемые форматы ссылок:
  - oz/1736756863
  - wb/145700662
  - ym/1732949807-100352880819-181725190
  - mm/100065768905
  - Полные URL маркетплейсов

* Рекомендуется использовать короткий вариант записи. Получить короткий вариант записи ссылок можно методом /valid-order.

* В большинстве случаев короткий вариант записи состоит из символа маркетплейса и ID (SKU) товара, но для некоторых маркетплейсов вторая часть может иметь другой формат записи.

* Информацию по доступным маркетплейсам и их символы можно получить используя метод /markets

**Параметры прокси:**
- Формат записи: USERNAME:PASSWORD@HOST:PORT
- Можно указать несколько прокси-серверов

**Особенности:**
- При успешной обработке возвращается order_hash
- order_hash используется для отслеживания статуса выполнения заказа
- Заказ проходит валидацию
- Количество товаров ограничено лимитом токена
- Использование proxyPool и cookies для обхода блокировок

**Заголовок запроса:**
- Authorization: Bearer <YOUR-TOKEN>

```python
import requests

headers = {
    "Authorization": "Bearer your-token-here"
}

order_data = {
    "products": [
        "oz/1596079870",
        "wb/300365052",
        "ym/1732949807-100352880819-5997015",
        "mm/100028286032",
        "https://www.wildberries.ru/catalog/95979396/detail.aspx",
    ],
    "proxyPool": [],
    "cookies": []
}

response = requests.post("http://domain/api/v1/order", json=order_data, headers=headers)
order_hash = response.text
print(f"Order hash: {order_hash}")
```
"#,
    request_body(
        content = Order, content_type = "application/json", description = "Заказ на парсинг",
        example = json!(
            {
                "products": [
                    "oz/1596079870", "wb/300365052", "ym/1732949807-100352880819-5997015", "mm/100028286032",
                    "https://www.ozon.ru/product/nozhnitsy-kantselyarskie-21-sm-calligrata-nerzhaveyushchaya-stal-plastik-173091046/",
                    "https://www.wildberries.ru/catalog/95979396/detail.aspx",
                    "https://market.yandex.ru/product--igrovaia-pristavka-sony-playstation-5-slim-digital-edition-bez-diskovoda-1000-gb-ssd-2-geimpada-bez-igr-belyi/925519649?sku=103706885579&uniqueId=162025048",
                    "https://megamarket.ru/catalog/details/nabor-instrumentov-v-keyse-108-predmetov-100065768905/"
                ],
                "proxy_pool": [],
                "cookies": []
            }
        )
    ),
    security(
        ("Token" = [])
    ),
    responses(
        (
            status = 200, description = "order_hash по которому можно получить статус выполнения задачи",
            content_type = "text/plain",
            example = "1a986959ef3b7fff2a16d774d3c56a9624d19d1d"
        ),
        (status = 400, description = r#"
### Ошибка ApiError

Значения ошибок смотреть в таблице ApiError
"#,
		body = ApiError, content_type = "application/json",
        example = json!({"error":"Unknown","code":0,"message":"Unknown server error."}))
    )
)]
#[allow(dead_code)]
fn order() {}

#[utoipa::path(
    post,
    path = "/valid-order",
    tags = ["order"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### GET POST /valid-order
Метод валидации данных заказа перед его отправкой.

**Описание:**
Позволяет проверить корректность данных заказа без его фактического создания в системе.

**Особенности:**
- Проверяет структуру и формат данных заказа
- Не требует авторизации
- Может изменять структуру заказа
- Возвращает провалидированный заказ

**Заголовок запроса:**
- Authorization: Bearer <YOUR-TOKEN>

```python
import requests

headers = {
    "Authorization": "Bearer your-token-here"
}
order_data = {
    "products": [
        "oz/1736756863",
        "wb/145700662"
    ],
    "proxyPool": [
        "user1:pass1@host1:port1"
    ],
    "cookies": []
}

response = requests.get("http://domain/api/v1/valid-order", headers=headers, json=order_data)
validated_order = response.json()
print(validated_order)
```
"#,
    request_body(
        content = Order, content_type = "application/json", description = "Заказ до валидации",
        example = json!(
            {
                "products": [
                    "oz/1596079870", "wb/300365052", "ym/1732949807-100352880819-5997015", "mm/100028286032",
                    "https://www.ozon.ru/product/nozhnitsy-kantselyarskie-21-sm-calligrata-nerzhaveyushchaya-stal-plastik-173091046/",
                    "https://www.wildberries.ru/catalog/95979396/detail.aspx",
                    "https://market.yandex.ru/product--igrovaia-pristavka-sony-playstation-5-slim-digital-edition-bez-diskovoda-1000-gb-ssd-2-geimpada-bez-igr-belyi/925519649?sku=103706885579&uniqueId=162025048",
                    "https://megamarket.ru/catalog/details/nabor-instrumentov-v-keyse-108-predmetov-100065768905/"
                ],
                "proxyPool": [],
                "cookies": []
            }
        )
    ),
    security(
        ("Token" = [])
    ),
    responses(
        (
            status = 200, description = "Заказ после валидации", body = Order, content_type = "application/json",
            example = json!({
                    "products": [
                    "oz/1596079870",
                    "wb/300365052",
                    "ym/1732949807-100352880819-5997015",
                    "mm/100028286032",
                    "oz/173091046",
                    "wb/95979396",
                    "ym/925519649-103706885579-162025048",
                    "mm/100065768905"
                    ],
                    "proxyPool": [],
                    "cookies": []
            })
        ),
        (status = 400, description = r#"
### Ошибка ApiError

Значения ошибок смотреть в таблице ApiError
"#,
		body = ApiError, content_type = "application/json",
        example = json!({"error":"Unknown","code":0,"message":"Unknown server error."}))
    )
)]
#[allow(dead_code)]
fn valid_order() {}

#[utoipa::path(
    get,
    path = "/task/{order_hash}",
    tags = ["order"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### GET /task/{order_hash}
Метод получения информации о состоянии задачи по её order_hash.

**Параметры пути:**
- order_hash: Уникальный идентификатор заказа (получается после отправки заказа методом /order)

**Заголовок запроса:**
- Authorization: Bearer <YOUR-TOKEN>

```python
import requests
import time

headers = {
    "Authorization": "Bearer your-token-here"
}

order_hash = "your-order-hash"
response = requests.get(f"http://domain/api/v1/task/{order_hash}", headers=headers)
task = response.json()

# Ожидание завершения задачи
while task["status"] in ("waiting", "processing"):
    response = requests.get(f"http://domain/api/v1/task/{order_hash}", headers=headers)
    task = response.json()
    print(task)
    time.sleep(1)
```

Для отслеживания выполнения задачи рекомендуется использовать подключение через WebSocket.
"#,
    params(
        (
			"order_hash" = String, Path,
			description = r#"order_hash заказа"#
		),
    ),
    security(
        ("Token" = [])
    ),
    responses(
        (
            status = 200, description = "Статус задачи", body = Task, content_type = "application/json",
            example = json!(
                {"queueNum":0,"status":"completed","progress":[3,3],"result":{"data":{"oz/1596079870":{"sku":"1596079870","name":"Xiaomi Телевизор TV A 43\" FHD 2025 43\" Full HD, черный","url":"https://www.ozon.ru/product/1596079870","price":23990,"cprice":23750,"seller":"Ozon Express","sellerId":"supermarket-25000","img":"https://cdn1.ozone.ru/s3/multimedia-1-1/7046147689.jpg","reviews":3307,"rating":4.8,"brand":"Xiaomi"},"oz/1793879666":{"sku":"1793879666","name":"Poco Смартфон POCO X7 Global 8/256 ГБ, черный","url":"https://www.ozon.ru/product/1793879666","price":23147,"cprice":22053,"seller":"FG Store","sellerId":"1076935","img":"https://cdn1.ozone.ru/s3/multimedia-1-s/7262525512.jpg","reviews":0,"rating":0.0},"wb/145700662":{"sku":"145700662","name":"Гель для стирки 5 литров для белого и цветного белья","url":"https://www.wildberries.ru/catalog/145700662/detail.aspx","price":491,"cprice":481,"seller":"ARIС Официальный магазин","sellerId":"1124859","reviews":150228,"rating":4.9,"brand":"ARIC"}}},"createdAt":1736857399}
            )
        ),
        (status = 400, description = r#"
### Ошибка ApiError

Значения ошибок смотреть в таблице ApiError
"#,
		body = ApiError, content_type = "application/json",
        example = json!({"error":"Unknown","code":0,"message":"Unknown server error."}))
    )
)]
#[allow(dead_code)]
fn task() {}

#[utoipa::path(
    get,
    path = "/task-ws/{order_hash}",
    tags = ["order"],
    context_path = &*ROOT_API_PATH,
    description = r#"
### ANY /task-ws/{order_hash}
Метод установки WebSocket-соединения для получения обновлений о состоянии задачи в реальном времени.

**Описание:**
Позволяет установить постоянное соединение для мониторинга статуса выполнения заказа.
WebSocket проверяет статус выполнения задачи и отправляет её клиенту в случае изменения статуса.

**Параметры пути:**
- order_hash: Уникальный идентификатор заказа (получается после отправки заказа методом /order)

**Особенности:**
- Использует протокол "send-only"
- Количество одновременных соединений ограничено
- Требует авторизации
- Автоматически закрывается после завершения задачи

**Заголовок запроса:**
- Authorization: Bearer <YOUR-TOKEN>

```python
from websockets.sync.client import connect

order_hash = "your-order-hash"
headers = {
    "Authorization": f"Bearer {TOKEN}"
}

with connect(f"ws://domain/api/v1/task-ws/{order_hash}", additional_headers=headers) as task-ws:
    try:
        while (task := task-ws.recv()):
            print(task)
    except Exception as e:
        print("Connection closed...")
```
"#,
    params(
        ("order_hash" = String, Path, description = "order_hash заказа"),
    ),
    security(
        ("Token" = [])
    ),
    responses(
        (
            status = 200, description = "Статус задачи", body = Task, content_type = "application/json",
            example = json!(
                {"queueNum":0,"status":"completed","progress":[3,3],"result":{"data":{"oz/1596079870":{"sku":"1596079870","name":"Xiaomi Телевизор TV A 43\" FHD 2025 43\" Full HD, черный","url":"https://www.ozon.ru/product/1596079870","price":23990,"cprice":23750,"seller":"Ozon Express","sellerId":"supermarket-25000","img":"https://cdn1.ozone.ru/s3/multimedia-1-1/7046147689.jpg","reviews":3307,"rating":4.8,"brand":"Xiaomi"},"oz/1793879666":{"sku":"1793879666","name":"Poco Смартфон POCO X7 Global 8/256 ГБ, черный","url":"https://www.ozon.ru/product/1793879666","price":23147,"cprice":22053,"seller":"FG Store","sellerId":"1076935","img":"https://cdn1.ozone.ru/s3/multimedia-1-s/7262525512.jpg","reviews":0,"rating":0.0},"wb/145700662":{"sku":"145700662","name":"Гель для стирки 5 литров для белого и цветного белья","url":"https://www.wildberries.ru/catalog/145700662/detail.aspx","price":491,"cprice":481,"seller":"ARIС Официальный магазин","sellerId":"1124859","reviews":150228,"rating":4.9,"brand":"ARIC"}}},"createdAt":1736857399}
            )
        ),
        (status = 400, description = r#"
### Ошибка ApiError

Значения ошибок смотреть в таблице ApiError
"#,
		body = ApiError, content_type = "application/json",
        example = json!({"error":"Unknown","code":0,"message":"Unknown server error."}))
    )
)]
#[allow(dead_code)]
fn task_ws() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi() {
		std::fs::write(
			"openapi.json",
			serde_json::to_string_pretty(
				&ApiDoc::openapi()
			).unwrap()
		).unwrap();
    }
}
