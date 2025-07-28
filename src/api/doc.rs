use std::sync::LazyLock;

use serde::Serialize;
use utoipa::{
    openapi::{
        self,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
    Modify, OpenApi,
};

use super::{
    super::{
        api::app::ROOT_API_PATH,
        config::{self as cfg, Config},
        models::{
            api::{ApiState, Order, Task, Token},
            scraper::Market,
        },
    },
    error::ApiError,
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

pub static API_DESCRIPTION: LazyLock<String> = LazyLock::new(|| {
    if let Some(path) = &cfg::get().api.description_file_path {
        match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => DEFAULT_API_DESCRIPTION.into(),
        }
    } else {
        DEFAULT_API_DESCRIPTION.into()
    }
});

const DEFAULT_API_DESCRIPTION: &'static str = r#"
Дата публикации: **1/19/25**

# Документация API

[Github page](https://github.com/Nikita55612/RustScraperApi)

[Python клиент](https://pypi.org/project/pyRustScraperApi/)

---

## О проекте

**RustScraperApi** — это высокопроизводительное API для сбора данных о товарах с популярных маркетплейсов, разработанное на языке программирования [Rust](https://ru.wikipedia.org/wiki/Rust_(%D1%8F%D0%B7%D1%8B%D0%BA_%D0%BF%D1%80%D0%BE%D0%B3%D1%80%D0%B0%D0%BC%D0%BC%D0%B8%D1%80%D0%BE%D0%B2%D0%B0%D0%BD%D0%B8%D1%8F)). Оно спроектировано для работы в условиях высокой нагрузки, обеспечивая надежность и максимальную скорость работы.

С помощью RustScraperApi вы можете разрабатывать автоматизированные системы для мониторинга и управления ценами на товары. Этот инструмент идеально подходит для сбора статистики, анализа и решения широкого спектра задач, соответствующих вашим бизнес-целям.

Контакт разработчика:

- Telegram: [@Nikita5612](https://t.me/Nikita5612)

Полный доступ к сервису предоставляется на коммерческой основе. Подробности можно узнать в личных сообщениях.

Доступ для тестирования предоставляется бесплатно, однако он имеет ограничения по времени использования и лимитам заказа.

---

## Основные возможности

- Поддержка крупнейших маркетплейсов:
  - [Wildberries](https://www.wildberries.ru/)
  - [Ozon](https://www.ozon.ru/)
  - [Яндекс.Маркет](https://market.yandex.ru/)
  - [МегаМаркет](https://megamarket.ru/)
- Система обхода блокировок через прокси-серверы
- Поддержка пользовательских cookies для передачи настроек сессии и авторизации
- [WebSocket](https://ru.wikipedia.org/wiki/WebSocket) подключение для отслеживания статуса парсинга в реальном времени
- Система очередей и параллельная обработка нескольких заказов
- Простой и понятный REST API интерфейс

---

## Начало работы

### 1. Получение токена доступа

Для работы с API требуется токен доступа. На период тестирования токен можно получить с помощью метода `/test-token`. Тестовый токен предоставляется для уникальных IP-адресов.

Каждый токен имеет следующие ограничения:
- Лимит на количество товаров в заказе
- Лимит на количество одновременных обработок (обработка нескольких заказов параллельно)
- Ограничение времени жизни токена (TTL)

Если вам нужен доступ на более длительный срок или с расширенными лимитами, свяжитесь со мной ([@Nikita5612](https://t.me/Nikita5612)) для уточнения условий и стоимости.

### 2. Составление заказа на парсинг товаров

Заказ на парсинг состоит из трех основных компонентов:
- Список товаров (`products`)
- Пул прокси-серверов (`proxyPool`)
- Пользовательские cookies (`cookies`)

```json
{"products": [], "proxyPool": [], "cookies": []}
```

#### Форматы ссылок на товары

Поддерживается два формата указания товаров:

1. Короткий формат: `символ маркетплейса/уникальный идентификатор товара`
   - `wb/145700662` ([Wildberries](https://www.wildberries.ru/))
   - `oz/1736756863` ([Ozon](https://www.ozon.ru/))
   - `ym/1732949807-100352880819-5997015` ([Яндекс.Маркет](https://market.yandex.ru/))
   - `mm/100065768905` ([МегаМаркет](https://megamarket.ru/))

2. Полный URL товара с маркетплейса

---

### 3. Отправка заказа и получение результатов

Процесс парсинга состоит из следующих шагов:
1. Отправка заказа методом [/order](#/order/order)
2. При успешной обработке возвращается `order_hash` для отслеживания статуса выполнения
3. Мониторинг выполнения через REST API или [WebSocket](https://ru.wikipedia.org/wiki/WebSocket)

### REST API мониторинг

Получение статуса выполнения через периодические запросы к методу `/task/{order_hash}`

### WebSocket мониторинг (рекомендуется)

Установка постоянного соединения через `/task-ws/{order_hash}` для получения обновлений в реальном времени. WebSocket отправляет статус выполнения задачи только в случае ее изменения.

---

## Результат парсинга

Резальтаты парсинга передаются в структуре Task. Она хранит в себе текущий статус выполнения и результаты парсинга. Результат может быть `data` или `error`. Если заказа начал выполнятся в `data` будут записаны результаты парсинга в структуре HashMap.

#### Task

```json
{
    ...
    "result": {
        "data": { "wb/145700662": {...}, "oz/1736756863": null, ...}
    }
}
```

Результат парсинга товара может быть представлен как `ProductData`, либо вернуть `null`. Это связано с возможными сбоями в процессе парсинга, такими как:

- Длительное время загрузки страницы.
- Загрузка страницы, не соответствующей ожиданиям (например, сообщение о блокировке или капча).
- Ошибки в работе парсера.

Эти ситуации неизбежны при работе с динамическими ресурсами, поэтому иногда не удается получить данные о товаре. Ошибки часто возникают из-за блокировки со стороны ресурса. Для обхода блокировок рекомендуется использовать прокси-серверы.

### Данные о товаре

`ProductData` представляет собой структуру данных, используемую для описания товара. Она содержит информацию о товаре, включая его идентификатор, название, цену, продавца и другие атрибуты.

### Поля

- **sku** (`string`)
  Уникальный идентификатор товара.

- **url** (`string`)
  Ссылка на товар.

- **name** (`null | string`)
  Название товара.

- **price** (`null | int64`)
  Цена товара.

- **cprice** (`null | int64`)
  Цена товара по карте.

- **seller** (`null | string`)
  Имя продавца.

- **sellerId** (`null | string`)
  Идентификатор продавца.

- **img** (`null | string`)
  URL изображения товара.

- **reviews** (`null | int64`)
  Количество отзывов о товаре.

- **rating** (`null | float`)
  Рейтинг товара.

- **brand** (`null | string`)
  Бренд товара.

---

## Особенности работы

На большинстве маркетплейсов встроена защита от парсинга — частые запросы блокируются по IP-адресу. Чтобы избежать блокировки, используйте прокси-серверы. Они подменяют IP-адрес парсера, распределяя запросы между разными адресами, что помогает обходить ограничения и снижает риск блокировок.

### Система защиты от блокировок

API предоставляет два основных механизма защиты:

1. **Прокси-пул (ProxyPool)**:
   - Позволяет распределять запросы через разные IP-адреса.
   - Поддерживает формат подключения: `USERNAME:PASSWORD@HOST:PORT`.
   - Возможность указания нескольких прокси-серверов.
   - Геолокация прокси может влиять на доступность товаров и их цены, которые зависят от региона.

Отсутствие ProxyPool может привести к блокировке запросов из-за превышения лимита обращений с одного IP-адреса (сервера парсера).

2. **Пользовательские Cookies**:
   - Сохраняют авторизационные данные и настройки пользователя.
   - Позволяют учитывать выбранные пункты выдачи заказов.
   - Поддерживают передачу геолокационных данных, влияющих на отображение цен и наличие товаров.

Cookies содержат параметры пользовательской сессии, которые можно использовать для повышения эффективности парсинга. Например, передача данных авторизованного аккаунта через Cookie снижает вероятность блокировки. Кроме того, Cookie могут содержать информацию о выбранном пункте выдачи, что важно для корректного отображения цен, привязанных к данному региону.

Для сбора cookies с веб-страниц я разработал расширение для Google Chrome — **[CookieReaderExtension](https://github.com/Nikita55612/CookieReaderExtension/)**. Оно позволяет извлекать все cookies с текущей страницы и сохранять их в валидном формате для парсинга.

#### Комбинирование методов

Использование Cookies** совместно с ProxyPool позволяет гибко управлять параметрами запросов и минимизировать риск блокировок.

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
| **WebSocketLimitExceeded** | Невозможно установить новое WebSocket-соединение,</br>так как сервер достиг максимального лимита одновременных подключений | **304** | 409 |
| **AccessRestricted** | Доступ к методу ограничен | **305** | 409 |
| **TokenDoesNotExist** | Токен не существует | **400** | 404 |
| **TaskNotFound** | Задача с указанным order_hash не существует | **401** | 404 |
| **PathNotFound** | Запрошенный путь не найден | **404** | 404 |
| **TaskSendFailure** | Не удалось отправить задачу обработчику | **500** | 500 |
| **ReqwestSessionError** | Ошибка сессии запроса | **501** | 500 |
| **DatabaseError** | Сбой транзакции базы данных | **502** | 500 |
| **SerializationError** | Не удалось сериализовать объект | **503** | 500 |
</br>

---
"#;

//////////////////////////////////
// API Documentation Structure  //
////////////////////////////////
#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "https://rustscraper.ru", description = "Remote API https")
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

response = requests.get("https://rustscraper.ru/api/token-info", headers=headers)
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
response = requests.get(f"https://rustscraper.ru/api/token-info/{token_id}")
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

Метод позволяет получить ограниченный по времени и лимитам токен для тестирования функциональности API.

**Параметры токена:**
- **id** - Строка токена для передачи в заголовок запроса
- **createdAt** - Дата и время создания токена в формате timestamp
- **ttl** - Время жизни токена в секундах
- **orderProductsLimit** - Лимит токена на количество товаров в заказе
- **taskCountLimit** - Лимит токена на количество параллельных обработок заказа

```python
import requests

response = requests.get("https://rustscraper.ru/api/test-token")
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
Позволяет отправить заказ на обработку с указанием списка товаров, настроек прокси-серверов и cookies.

**Формат данных:**
- В параметре products можно указывать как прямые ссылки на товары, так и короткий формат `символ маркетплейса/уникальный идентификатор товара`
- Поддерживаемые форматы ссылок:
  - oz/1736756863
  - wb/145700662
  - ym/1732949807-100352880819-181725190
  - mm/100065768905
  - Полные URL на страницу товара

* Рекомендуется использовать короткий вариант записи. Получить короткий вариант записи ссылок можно методом /valid-order.

* В большинстве случаев короткий вариант записи состоит из символа маркетплейса и ID (SKU) товара, но для некоторых маркетплейсов вторая часть может иметь другой формат записи.

* Информацию по доступным маркетплейсам и их символы можно получить используя метод /markets

**Параметры прокси:**
- Формат записи: USERNAME:PASSWORD@HOST:PORT
- Можно указать несколько прокси-серверов

Параметры proxyPool и cookies опциональны. Отсутствие proxyPool может привести к блокировке запросов из-за превышения лимита обращений с одного IP адреса (сервера парсера).

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

response = requests.post("https://rustscraper.ru/api/order", json=order_data, headers=headers)
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
Позволяет проверить корректность данных заказа без его фактического создания в системе. Метод возвращает провалидированный заказ в случае успеха или ошибку если заказ не прошел валидацию.

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
        "wb/145700662",
        "rt/id88888888" # Ошибка
    ],
    "proxyPool": [
        "user1:pass1@host1:port1",
        "pass1@host1:port1" # Ошибка
    ],
    "cookies": []
}

response = requests.get("https://rustscraper.ru/api/valid-order", headers=headers, json=order_data)
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
response = requests.get(f"https://rustscraper.ru/api/task/{order_hash}", headers=headers)
task = response.json()

# Ожидание завершения задачи
while task["status"] in ("waiting", "processing"):
    response = requests.get(f"https://rustscraper.ru/api/task/{order_hash}", headers=headers)
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

С подключением через WebSocket сервер сам проверяет статус заказа и отправляет его в случае изменения.

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

with connect(f"ws://rustscraper.ru/api/task-ws/{order_hash}", additional_headers=headers) as task-ws:
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
            serde_json::to_string_pretty(&ApiDoc::openapi()).unwrap(),
        )
        .unwrap();
    }
}
