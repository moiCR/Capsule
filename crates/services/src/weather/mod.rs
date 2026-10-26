use arc_swap::ArcSwap;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeatherStatus {
    pub temp_c: i32,
    pub description: String,
    pub icon: &'static str,
    pub city: String,
}

#[derive(Deserialize)]
struct WttrResponse {
    current_condition: Option<Vec<WttrCondition>>,
    nearest_area: Option<Vec<WttrArea>>,
}

#[derive(Deserialize)]
struct WttrCondition {
    #[serde(rename = "temp_C")]
    temp_c: Option<String>,
    #[serde(rename = "weatherCode")]
    weather_code: Option<String>,
    #[serde(rename = "weatherDesc")]
    weather_desc: Option<Vec<WttrDesc>>,
}

#[derive(Deserialize)]
struct WttrDesc {
    value: Option<String>,
}

#[derive(Deserialize)]
struct WttrArea {
    #[serde(rename = "areaName")]
    area_name: Option<Vec<WttrDesc>>,
}

fn map_weather_code_to_icon(code: u32) -> &'static str {
    match code {
        113 => "sun.svg",
        116 => "cloud-sun.svg",
        119 | 122 => "cloud.svg",
        200 | 386 | 389 | 392 | 395 => "cloud-lightning.svg",
        176 | 263 | 266 | 281 | 284 | 293 | 296 | 299 | 302 | 305 | 308 | 311 | 314 | 353 | 356
        | 359 => "cloud-rain.svg",
        179 | 182 | 227 | 230 | 323 | 326 | 329 | 332 | 335 | 338 | 368 | 371 => "cloud-snow.svg",
        _ => "cloud.svg",
    }
}

#[derive(Clone)]
pub struct WeatherService {
    status: Arc<ArcSwap<Option<WeatherStatus>>>,
}

impl Default for WeatherService {
    fn default() -> Self {
        Self::new()
    }
}

impl WeatherService {
    pub fn new() -> Self {
        let status = Arc::new(ArcSwap::from_pointee(None));
        let status_clone = status.clone();

        crate::spawn_tokio(async move {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(6))
                .user_agent("curl/7.88.1")
                .build()
                .unwrap_or_default();

            loop {
                if let Ok(res) = client.get("https://wttr.in/?format=j1").send().await
                    && let Ok(data) = res.json::<WttrResponse>().await
                    && let Some(cond) = data.current_condition.and_then(|c| c.into_iter().next())
                {
                    let temp_c = cond
                        .temp_c
                        .and_then(|t| t.trim().parse::<i32>().ok())
                        .unwrap_or(0);
                    let code = cond
                        .weather_code
                        .and_then(|c| c.trim().parse::<u32>().ok())
                        .unwrap_or(113);
                    let icon = map_weather_code_to_icon(code);
                    let description = cond
                        .weather_desc
                        .and_then(|mut v| v.pop())
                        .and_then(|d| d.value)
                        .unwrap_or_default();
                    let city = data
                        .nearest_area
                        .and_then(|mut a| a.pop())
                        .and_then(|a| a.area_name)
                        .and_then(|mut n| n.pop())
                        .and_then(|d| d.value)
                        .unwrap_or_default();

                    status_clone.store(Arc::new(Some(WeatherStatus {
                        temp_c,
                        description,
                        icon,
                        city,
                    })));
                }

                tokio::time::sleep(Duration::from_secs(900)).await;
            }
        });

        Self { status }
    }

    pub fn get_status(&self) -> Option<WeatherStatus> {
        (**self.status.load()).clone()
    }
}
