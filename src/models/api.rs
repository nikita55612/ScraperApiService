#![allow(warnings)]
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::super::models::scraper::ProductData;
use super::super::utils::{
    gen_token_id,
    timestamp_now,
    sha1_hash
};


type OrderHash = String;

#[derive(Clone, Debug, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub struct Token {
    pub id: String,

	#[serde(rename="createdAt")]
    pub created_at: u64,
    pub ttl: u64,
    pub ilimit: u64
}

impl Token {
    pub fn new(ttl: u64, ilimit: u64) -> Self  {
        Self {
            id: gen_token_id(),
            created_at: timestamp_now(),
            ttl,
            ilimit
        }
    }

    pub fn is_expired(&self) -> bool {
        (self.created_at + self.ttl) - timestamp_now() < 0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all="lowercase")]
pub enum TaskStatus {
    Waiting,
    Processing,
    Completed,
    Error
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all="lowercase")]
pub enum TaskResult {
    Data(HashMap<String, Option<ProductData>>),
    Error(String)
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Order {
	#[serde(rename="tokenId")]
    pub token_id: String,
    pub products: Vec<String>,

	#[serde(rename="proxyList")]
    pub proxy_list: Vec<String>,
	#[serde(rename="cookieList")]
	pub cookie_list: Vec<OrderCookiesParam>,
}

impl Order {
    fn sha1_hash(&self) -> OrderHash {
        let order_hash_data = format!(
            "{} {}",
            self.token_id,
            self.products.join(",")
        );

        sha1_hash(order_hash_data.as_bytes())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    #[serde(skip)]
    pub order: Order,
    #[serde(skip)]
    pub order_hash: OrderHash,

    #[serde(rename="queueNum")]
    pub queue_num: u64,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<TaskProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<TaskResult>,
    pub created: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskProgress(u64, u64);

impl TaskProgress {
    pub fn new(done: u64, total: u64) -> Self {
        Self (done, total)
    }

    pub fn next_step(&mut self) {
        self.0 += 1;
    }
}

impl Task {
    pub fn from_order(order: Order) -> Self {
        let order_hash = order.sha1_hash();
        Self {
            order: order,
            order_hash,
            queue_num: 0,
            status: TaskStatus::Waiting,
            progress: None,
            result: None,
            created: timestamp_now(),
        }
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status
    }

    pub fn init_result_data(&mut self) {
        self.result = Some(
            TaskResult::Data(
                HashMap::new()
            )
        )
    }

    pub fn insert_result_item(&mut self, k: String, v: Option<ProductData>) {
        if let Some(
            TaskResult::Data(items_map)
        ) = &mut self.result {
            items_map.insert(k, v);
        }
    }

    pub fn set_progress(&mut self, done: u64, total: u64) {
        self.progress = Some(TaskProgress::new(done, total));
    }

    pub fn init_progress(&mut self) {
        let total = self.order.products.len() as u64;
        self.set_progress(0, total);
    }

    pub fn next_progress_step(&mut self) {
        if let Some(progress) = self.progress.as_mut() {
            progress.next_step();
        }
    }

    pub fn get_curr_step(&self) -> u64 {
        if let Some(TaskProgress(done, _)) = &self.progress {
            return *done;
        }

        0
    }

    pub fn is_done(&self) -> bool {
        if let Some(progress) = &self.progress {
            return progress.0 == progress.1;
        }

        false
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderCookiesParam {
    name: String,
    value: String,
    url: String,
    domain: String,
    path: String,
    http_only: bool,
    same_site: String,
    secure: bool
}

/*
[{"domain":".chatgpt.com","http_only":false,"name":"oai-did","path":"/","same_site":"lax","secure":false,"value":"61c2471f-41d2-4c05-9ca2-1e5c35025fd6","url":"https://chatgpt.com"},{"domain":"chatgpt.com","http_only":false,"name":"oai-nav-state","path":"/","same_site":"lax","secure":false,"value":"1","url":"https://chatgpt.com"},{"domain":"chatgpt.com","http_only":false,"name":"oai-hlib","path":"/","same_site":"lax","secure":false,"value":"true","url":"https://chatgpt.com"},{"domain":"chatgpt.com","http_only":false,"name":"oai-locale","path":"/","same_site":"lax","secure":false,"value":"en-US","url":"https://chatgpt.com"},{"domain":"chatgpt.com","http_only":true,"name":"__Host-next-auth.csrf-token","path":"/","same_site":"lax","secure":true,"value":"ace6db50191733c074b68c276992ae287a55236f1d942a160017b61c20d3df08%7C0ffc180bcd68c2e32fcae7d7c8a9afa16341b487f3be9d7298d97dff03d06002","url":"https://chatgpt.com"},{"domain":"chatgpt.com","http_only":true,"name":"__Secure-next-auth.callback-url","path":"/","same_site":"lax","secure":true,"value":"https%3A%2F%2Fchat-onramp.unified-6.api.openai.com","url":"https://chatgpt.com"},{"domain":".chatgpt.com","http_only":true,"name":"__Secure-next-auth.session-token","path":"/","same_site":"lax","secure":true,"value":"eyJhbGciOiJkaXIiLCJlbmMiOiJBMjU2R0NNIn0..Gpq7d3bQkwEzzWCw.k4Ca9z76F96pYQ15O8jInM55DhC776Tnhb0sSU6ANz7vtl2HCMb5BaNJSZwtVCdqLrlsCaT_VKS_bNGu3DkFfp_CKvqto0Ekb2CQQJLBp__Syxn8_1f6LsM7617Yi-kZOPe8KIf3s14tVlcQ4lCZ3JKy7TfJFUihmmmbHQBrRT5gqQhI_COijuZ_HxTBW7X5pu2clxBITp5GPFHZbU6eQYMiy_YaQW-e6kTQ9ExQbRQFCbB_AyEY4yBxL5Ou7Yboio_j4XIlHZkXFVFn6PKBgNgM9l9-E2u97RqRZiMeaas3TdKrs9yDVbd8VK54ZQc5be4Y_Yp3vHPAuVsgIJpsgghk3SlPW0vXnBsAPKJ5QDW_oTYopgI9R34ecPV6uHwawuYexaU3jNkNEQf9BVWfELuhfG5eP7Ru3ZJ4sRs-N8MCyDQjKLOine6JaXiPGZli-SZmzC4df38_BZ5qta6lpZpsWW5o20O5P8Qk3MeyF5Q58ibK1P0T8mzxn9jm44ekk5Gc0JGS1H3is9eDWteTDUo6bdhO5o8TovKnASmQLjgC5diQilzyGD9HvlboQDocrRj-n_MLsEMSFCTEBKYK1rTclmcu24kykS-UNLRpa850tY9Cy2pbUqO07RSUvnh8KqS0CywKcV9j1oTs31mMxDSgbQgbAOSAH8C9rqPJLLdSAsP-SG81C3A95eNgr-VCel5Tz5mnxgwyDsnA9zrAoxBpPpgC847936gl0MTVQKXWR2sxyL4_Yq8h-ULTU9C5L2YE2gW_aJ-lOoagkcUt7j1n7DBhLFBwXGRAmlP8rT_29y09oOCSDlCoLAOBfH2M5pgoyOXvlIzaPqNxcE_nVo_TP-_-mho-Q_loE9fpYwxs0eG2AuMUX3-fD5YCHmoxw0vC2YmSXDBAPTGnh6Xhd_XY7dlu835MC0lfZQY4jletdVAU6b3NbXrM91UiIl1DOrcQMs_N_EdDHF9rJD7Ue9xQauXLUOsxzepFNeaLNE80Psj8MaSWBTqRO3pb55QBZqUjLk6D3jm-zQ6-rus3bZPBtIsEni2YX0vbdwp21R8buW1f9a3QlFJB8EsMLaglbH5T4bfHNwKrX1wYAon1EGNguQJNsFoGc7qeD5RK-9nykwpHHuVoEa6h6hCgsTPsc8W76g1VghMrl-6-7IhDCn2PjEsdQRGD2GoE1RErPRouSHWkV7Rx2WdZIf03e1j6EoGeyOk9pyncXxSgV3Jn44ZakA9c9ecLQ9ypap9YJ2zz8irENAdG6D6QH9FC-Y0ayowR0L7lOup_2db0Wk_MuRRcqDsALBBWlVok882lWXhIrkZvtjFcSALR8Kzh8JMd2PLvAb0Ob7s9cAz-KFPVlboPb99yFYlq7-eHkfT9nqtGAum5frXzoQUm24RkFIdZC6nvutRmyQi6I9LJ1lHPrRcYQzJZwo0V2VENEEXdxGqt-f5ZYE4zreQ7E7DUWfhskhpCOonDDbZY-OhRVB0EllxenUGcaQ_HDjBJUMyDSOpx91WqF2fkd-fQ4jCDLkZR-Q3ZyuyN2zP95m8WJT23GvficM_SfECmEh9lnUNsenPolqVQuPixkeTMflScU4rIaD-Cf8J3pI1w2oTV_7lzJFRSjN6aTZbYVlyB_AoRuucc4iPx9nQ6HpBZmf_C7TyeHYhSo8ABlcpyQkuJeDfImNuyCkIfnyPkAM9IH6OKlowlcp-2upenk8nfrgPaSu5c9ER6XXUCd0pzYqZBQbDTnnypcQMUxjrnXSwnmDRncJbj21MZKsJNklZQef03KrNxVvBNUlFOAnZ9SY9UcwAOdBL1jMha-gmZJW3xB4iXOq5hrylb3ubml4HiaNyXPphsj4ePLrdqdWXZ7v2ciCq58caAGIrcy0n-065RpnhL-Wnr5NzmgDYzEN3WVOD7WiTGeygC4NLobAPGq3P0oJaeHyBYEDWiJodD12a7ZOd5TXsf-vMjj8NiWTheSryCPVd51pHVAXOVBxsaB08ogOhJ0xfI3l2lWMzff-o2W8TFyuXyUPUpN4G22UhrNJAwpnDRdrt8fEfXlcgSanQezCDWILbxVZ92fjyBDdr_sar9IWXViOvickwovE3IDTHvy_HmARDG53OxMpjgbA8yIBXurBuodjf3bL0T6jl2EDxte1ZLFFexrNV--Tp5MQKHs7RHq39O17v1oV1d9RtAtYq79MiT1V9yBM4X2twFgC6FmQ7Z9oIzLI_qTnIj_sUyf2MkSwfvajq2v8FHyzBdHr-GprY8VGmmqvLGK9dCDUBi5xs93g36SRNXBJeuDUPZ_W5WE41jN9W57KYJoCa_Vy2IjrCW7r6YY_qXiyo003GPDeQwM5GxeBVYxptLbBvKb7Jx_bERy3cTOdTNKZrz7K8FMeXhYs2DNZW5cpyDT9FCxreY0_JrBYYWktqDv_4e3SUg_Tn4J4zXjSM-e7MhxbzaqXESRJoCz2MW5xBqu-1lkTnWIIiuMOfemnyXYCIKQxmHYHn0_U83dle7afn3itW6bHVqVaW-8C3BaQqai-DKNT23JMqdbMrRqbbY8jIF40QocnDyJmbuP1EwaQnTuNlVGVzlIqZ1h8KgL54-t-c-PS8lfsAo-uDNf5hn0crNT0gIuT_cWMvFvnJbqwcT0DVK9I1oy4lnkwxc_1_UBPn4LxsvAHt0rwIuOF7AyhjstlBXuToDFnyKkwQNgUiAO1lEDkDrOdQcB_QUYzNBfHFsnB10I_Hx84hK-l3gI7YGYSWsjBir4I8Dvx9R-EeGj6M6Oncj6EUJnDFFYMdXJRvficEX8clKZe9wbShybvkQEM5bs8XbC2Q7JtJs392yRfkhniQywvSYSMCYNt7qReSWXfG4JyydHOe0mjNW7NQOFGXgoOvrorYCBn7kJZv1kSzg_k3u44D5h7_j3FC1G8YodfiAgjvcQU9wbmj4-rlhzczCaWJ7oqnNdLsd2yuhSBCodvrwh3hvugEw7rGzTn_CItiW1p-pw4b5g-cCde6QByvyojKC-4feOM0wHIIWewrTBLNKmRY08jCOQOocUCBGLlbnHVBB6PR5ZS6JDk-B9PVrB6xX-vEKpkV4CMRBcukecNhi3j48OiKxRB3YBCfm-aE5G6KuaEdRGt9AuHPOKca22-jbqaZHHHW_FLo0uKmO1_M1hGx-99MSwfGiQDmzKr7h7LhKQ9dUCHEcSM4r_CbP-6GiH9wlGU8FPAgAiJJOwO97cGS25EkgC5biHKW_pVOW2skGTB8ayEIifTWM7Qlr1t1rD4YCGJDwGNDP3AuUMYRH_OXdd8sJw45EVjnb8u68gs6LfjkM64R9ZokZjq6se41KqBkTqNalr_s.11vGJfkIdFAvd6o8yf8GLQ","url":"https://chatgpt.com"},{"domain":".chatgpt.com","http_only":true,"name":"_cfuvid","path":"/","same_site":"no_restriction","secure":true,"value":"Irsr1f9tGc8wjxNTeM9jLTFL.v7QVaLP.ZmJ8dztniE-1735061838090-0.0.1.1-604800000","url":"https://chatgpt.com"},{"domain":"chatgpt.com","http_only":false,"name":"oai-sc","path":"/","same_site":"no_restriction","secure":true,"value":"0gAAAAABnaxdSlkQ9x7BOxgv1s2fP2N53B3M9yvlDOgzjyomkWlxJ_XAss3WH4F471IUdXnyKSTCcpMETALI_W0C_wypBfoolm35lHFUrxAxP9Dk5j9O8Gydlj4q3I1SPc6Rd68eU8DeEoi4hwY2aVIPSGKc8eTIkIHQZ62RyBNxOVZr06cfIYx_ZzV8qp76tqboTq3CQKoI1XyKV_s7IszI9w0GPwR4u55rxMhjh_mBqu2taIlOlO_w","url":"https://chatgpt.com"},{"domain":"chatgpt.com","http_only":false,"name":"_dd_s","path":"/","same_site":"strict","secure":false,"value":"isExpired=1","url":"https://chatgpt.com"}]
 */
