pub mod model;

pub(crate) type NCMResult<T> = Result<T, Errors>;
use super::netease::encrypt::Crypto;
use super::SongTag;
use lazy_static::lazy_static;
use model::*;
use openssl::hash::{hash, MessageDigest};
use rand::thread_rng;
use regex::Regex;
use reqwest::blocking::Client;
use std::{collections::HashMap, time::Duration};

lazy_static! {
    static ref _CSRF: Regex = Regex::new(r"_csrf=(?P<csrf>[^(;|$)]+)").unwrap();
}

static BASE_URL_SEARCH: &str = "https://m.music.migu.cn/migu/remoting/scr_search_tag?";
static URL_LYRIC: &str = "https://music.migu.cn/v3/api/music/audioPlayer/getLyric?";

pub struct MiguApi {
    client: Client,
    #[allow(dead_code)]
    csrf: String,
}

impl MiguApi {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            // .cookies()
            .build()
            .expect("Initialize Web Client Failed!");
        Self {
            client,
            csrf: String::new(),
        }
    }

    fn request(&mut self, _params: HashMap<&str, &str>) -> NCMResult<String> {
        let url = BASE_URL_SEARCH.to_string();
        self.client
            .get(&url)
            .send()
            .map_err(|_| Errors::NoneError)?
            .text()
            .map_err(|_| Errors::NoneError)
    }

    #[allow(unused)]
    pub fn search(
        &mut self,
        keywords: &str,
        types: u32,
        offset: u16,
        limit: u16,
    ) -> NCMResult<String> {
        let result = self
            .client
            .get(BASE_URL_SEARCH)
            .header(
                "Referer",
                // format!(
                // "https://m.music.migu.cn/migu/l/?s=149&p=163&c=5111&j=l&keyword={}",
                // keywords.to_string()
                // ),
                "https://m.music.migu.cn",
            )
            .query(&[
                ("keyword", keywords.to_string()),
                ("pageNo", offset.to_string()),
                ("pageSize", limit.to_string()),
                ("type", 2.to_string()),
            ])
            .send()
            .map_err(|_| Errors::NoneError)?
            .text()
            .map_err(|_| Errors::NoneError)?;

        match types {
            1 => to_song_info(result, Parse::SEARCH).and_then(|s| Ok(serde_json::to_string(&s)?)),
            _ => Err(Errors::NoneError),
        }
    }

    // search and download lyrics
    // music_id: 歌曲id
    #[allow(unused)]
    pub fn song_lyric(&mut self, music_id: String) -> NCMResult<String> {
        let result = self
            .client
            .get(URL_LYRIC)
            .header("Referer", "https://m.music.migu.cn")
            .query(&[("copyrightId", music_id)])
            .send()
            .map_err(|_| Errors::NoneError)?
            .text()
            .map_err(|_| Errors::NoneError)?;

        to_lyric(result)
    }

    // 歌曲 URL
    // ids: 歌曲列表
    #[allow(unused)]
    pub fn songs_url(&mut self, id: String) -> NCMResult<Vec<SongUrl>> {
        let url = "http://media.store.kugou.com/v1/get_res_privilege";

        let kg_mid = Crypto::hex_random_bytes(4);
        let kg_mid_md5 = hex::encode(hash(MessageDigest::md5(), kg_mid.as_bytes())?);
        let kg_mid_string = format!("kg_mid={}", kg_mid_md5);
        println!("{}", kg_mid_string);

        let char_collection = b"abcdefghijklmnopqrstuvwxyz1234567890";
        let mut rng = thread_rng();

        // String:

        let mut result = String::new();
        let mut params = HashMap::new();
        params.insert("relate", 1.to_string());
        params.insert("userid", "0".to_string());
        params.insert("vip", 0.to_string());
        params.insert("appid", 1000.to_string());
        params.insert("token", "".to_string());
        params.insert("behavior", "download".to_string());
        params.insert("area_code", "1".to_string());
        params.insert("clientver", "8990".to_string());
        let mut params_resource = HashMap::new();
        params_resource.insert("id", 0.to_string());
        params_resource.insert("type", "audio".to_string());
        params_resource.insert("hash", id);
        let params_resource_string = serde_json::to_string(&params_resource)?;
        params.insert("resource", params_resource_string);
        println!("{}", serde_json::to_string(&params)?);

        let result = self
            .client
            .post(url)
            .header("Cookie", kg_mid_string)
            .header(
                "Referer",
                "http://www.kugou.com/webkugouplayer/flash/webKugou.swf",
            )
            .json(&params)
            .send()
            .map_err(|_| Errors::NoneError)?
            .text()
            .map_err(|_| Errors::NoneError)?;

        to_song_url(result)
    }

    // 歌曲详情
    // ids: 歌曲 id 列表
    #[allow(unused)]
    pub fn songs_detail(&mut self, ids: &[u64]) -> NCMResult<Vec<SongTag>> {
        let path = "/weapi/v3/song/detail";
        let mut params = HashMap::new();
        let c = format!(
            r#""[{{"id":{}}}]""#,
            ids.iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );
        let ids = format!(
            r#""[{}]""#,
            ids.iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );
        params.insert("c", &c[..]);
        params.insert("ids", &ids[..]);
        let result = self.request(params)?;
        to_song_info(result, Parse::USL)
    }
}