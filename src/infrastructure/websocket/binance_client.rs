use wasm_bindgen::prelude::*;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::http::Request;
use serde::Deserialize;
use futures::StreamExt;
use crate::domain::{
    market_data::{
        entities::{Candle, OHLCV},
        value_objects::{Price, Volume, Timestamp, Symbol, TimeInterval},
    },
    logging::{LogComponent, get_logger},
};

/// Binance WebSocket клиент на основе gloo
#[derive(Clone)]
pub struct BinanceWebSocketClient {
    symbol: Symbol,
    interval: TimeInterval,
}

#[derive(Debug, Deserialize)]
struct BinanceKlineData {
    #[serde(rename = "k")]
    kline: KlineInfo,
}

#[derive(Debug, Deserialize)]
struct KlineInfo {
    #[serde(rename = "t")]
    open_time: u64,
    #[serde(rename = "o")]
    open: String,
    #[serde(rename = "h")]
    high: String,
    #[serde(rename = "l")]
    low: String,
    #[serde(rename = "c")]
    close: String,
    #[serde(rename = "v")]
    volume: String,
}

/// Структура для исторических данных Binance Klines API
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BinanceHistoricalKline(
    u64,     // Open time
    String,  // Open
    String,  // High  
    String,  // Low
    String,  // Close
    String,  // Volume
    u64,     // Close time
    String,  // Quote asset volume
    u32,     // Number of trades
    String,  // Taker buy base asset volume
    String,  // Taker buy quote asset volume
    String,  // Ignore
);

impl BinanceWebSocketClient {
    pub fn new(symbol: Symbol, interval: TimeInterval) -> Self {
        Self {
            symbol,
            interval,
        }
    }

    /// Подключение к Binance WebSocket потоку
    pub async fn connect(&mut self) -> Result<WebSocket, String> {
        let symbol_lower = self.symbol.value().to_lowercase();
        let interval_str = self.interval.to_binance_str();
        
        let stream_name = format!("{}@kline_{}", symbol_lower, interval_str);
        let url = format!("wss://stream.binance.com:9443/ws/{}", stream_name);

        get_logger().info(
            LogComponent::Infrastructure("BinanceWS"),
            &format!("🔌 Connecting to Binance: {}", url)
        );

        let ws = WebSocket::open(&url)
            .map_err(|e| format!("Failed to open WebSocket: {:?}", e))?;
        
        get_logger().info(
            LogComponent::Infrastructure("BinanceWS"),
            &format!("✅ Connected to Binance stream: {}", stream_name)
        );

        Ok(ws)
    }

    /// Обработка сообщения от Binance
    pub fn parse_message(&self, data: &str) -> Result<Candle, String> {
        let kline_data: BinanceKlineData = serde_json::from_str(data)
            .map_err(|e| format!("Failed to parse Binance message: {}", e))?;

        let kline = &kline_data.kline;
        
        // Парсим цены
        let open = kline.open.parse::<f64>()
            .map_err(|_| "Invalid open price")?;
        let high = kline.high.parse::<f64>()
            .map_err(|_| "Invalid high price")?;
        let low = kline.low.parse::<f64>()
            .map_err(|_| "Invalid low price")?;
        let close = kline.close.parse::<f64>()
            .map_err(|_| "Invalid close price")?;
        let volume = kline.volume.parse::<f64>()
            .map_err(|_| "Invalid volume")?;

        // Создаем OHLCV
        let ohlcv = OHLCV::new(
            Price::new(open),
            Price::new(high),
            Price::new(low),
            Price::new(close),
            Volume::new(volume),
        );

        // Создаем свечу
        let candle = Candle::new(
            Timestamp::new(kline.open_time),
            ohlcv,
        );

        Ok(candle)
    }

    /// Запуск потока с обработчиком
    pub async fn start_stream<F>(&mut self, mut handler: F) -> Result<(), String>
    where
        F: FnMut(Candle) + 'static,
    {
        let mut stream = self.connect().await?;
        
        get_logger().info(
            LogComponent::Infrastructure("BinanceWS"),
            "🚀 Starting Binance WebSocket stream processing..."
        );

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(gloo_net::websocket::Message::Text(data)) => {
                    match self.parse_message(&data) {
                        Ok(candle) => {
                            get_logger().debug(
                                LogComponent::Infrastructure("BinanceWS"),
                                &format!(
                                    "📊 Received candle: {} - O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.2}",
                                    self.symbol.value(),
                                    candle.ohlcv.open.value(),
                                    candle.ohlcv.high.value(),
                                    candle.ohlcv.low.value(),
                                    candle.ohlcv.close.value(),
                                    candle.ohlcv.volume.value()
                                )
                            );
                            handler(candle);
                        },
                        Err(e) => {
                            get_logger().error(
                                LogComponent::Infrastructure("BinanceWS"),
                                &format!("❌ Failed to parse message: {}", e)
                            );
                        }
                    }
                },
                Ok(_) => {
                    // Игнорируем бинарные сообщения
                },
                Err(e) => {
                    get_logger().error(
                        LogComponent::Infrastructure("BinanceWS"),
                        &format!("❌ WebSocket error: {:?}", e)
                    );
                    break;
                }
            }
        }

        get_logger().info(
            LogComponent::Infrastructure("BinanceWS"),
            "🔌 WebSocket stream ended"
        );
        Ok(())
    }

    /// 📈 Загрузка исторических данных от Binance REST API
    pub async fn fetch_historical_data(&self, limit: u32) -> Result<Vec<Candle>, String> {
        let symbol_upper = self.symbol.value().to_uppercase();
        let interval_str = self.interval.to_binance_str();
        
        let url = format!(
            "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
            symbol_upper, interval_str, limit
        );

        get_logger().info(
            LogComponent::Infrastructure("BinanceAPI"),
            &format!("📈 Fetching {} historical candles from: {}", limit, url)
        );

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch historical data: {:?}", e))?;

        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let klines: Vec<BinanceHistoricalKline> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;

        let mut candles = Vec::new();
        
        for kline in klines {
            let open = kline.1.parse::<f64>()
                .map_err(|_| "Invalid open price")?;
            let high = kline.2.parse::<f64>()
                .map_err(|_| "Invalid high price")?;
            let low = kline.3.parse::<f64>()
                .map_err(|_| "Invalid low price")?;
            let close = kline.4.parse::<f64>()
                .map_err(|_| "Invalid close price")?;
            let volume = kline.5.parse::<f64>()
                .map_err(|_| "Invalid volume")?;

            let ohlcv = OHLCV::new(
                Price::new(open),
                Price::new(high),
                Price::new(low),
                Price::new(close),
                Volume::new(volume),
            );

            let candle = Candle::new(
                Timestamp::new(kline.0), // open_time
                ohlcv,
            );

            candles.push(candle);
        }

        get_logger().info(
            LogComponent::Infrastructure("BinanceAPI"),
            &format!("✅ Loaded {} historical candles for {}", candles.len(), symbol_upper)
        );

        Ok(candles)
    }

    /// 📈 Загрузка исторических данных до указанного времени
    pub async fn fetch_historical_data_before(&self, end_time: u64, limit: u32) -> Result<Vec<Candle>, String> {
        let symbol_upper = self.symbol.value().to_uppercase();
        let interval_str = self.interval.to_binance_str();

        let url = format!(
            "https://api.binance.com/api/v3/klines?symbol={}&interval={}&endTime={}&limit={}",
            symbol_upper, interval_str, end_time, limit
        );

        get_logger().info(
            LogComponent::Infrastructure("BinanceAPI"),
            &format!("📈 Fetching {} candles before {} from: {}", limit, end_time, url)
        );

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch historical data: {:?}", e))?;

        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let klines: Vec<BinanceHistoricalKline> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;

        let mut candles = Vec::new();

        for kline in klines {
            let open = kline.1.parse::<f64>()
                .map_err(|_| "Invalid open price")?;
            let high = kline.2.parse::<f64>()
                .map_err(|_| "Invalid high price")?;
            let low = kline.3.parse::<f64>()
                .map_err(|_| "Invalid low price")?;
            let close = kline.4.parse::<f64>()
                .map_err(|_| "Invalid close price")?;
            let volume = kline.5.parse::<f64>()
                .map_err(|_| "Invalid volume")?;

            let ohlcv = OHLCV::new(
                Price::new(open),
                Price::new(high),
                Price::new(low),
                Price::new(close),
                Volume::new(volume),
            );

            let candle = Candle::new(
                Timestamp::new(kline.0),
                ohlcv,
            );

            candles.push(candle);
        }

        get_logger().info(
            LogComponent::Infrastructure("BinanceAPI"),
            &format!("✅ Loaded {} historical candles", candles.len())
        );

        Ok(candles)
    }
}

/// Простая функция для создания WebSocket соединения
pub async fn create_binance_stream(symbol: &str, interval: &str) -> Result<BinanceWebSocketClient, String> {
    let symbol = Symbol::from(symbol);
    let interval = interval.parse::<TimeInterval>()
        .map_err(|_| format!("Invalid interval: {}", interval))?;
    
    let client = BinanceWebSocketClient::new(symbol, interval);
    Ok(client)
}

/// Экспортируемая функция для JavaScript
#[wasm_bindgen]
pub async fn test_binance_websocket() -> Result<(), JsValue> {
    get_logger().info(
        LogComponent::Infrastructure("BinanceWS"),
        "🧪 Testing Binance WebSocket with gloo..."
    );
    
    let mut client = create_binance_stream("BTCUSDT", "1m").await
        .map_err(|e| JsValue::from_str(&e))?;
    
    // Тестовый обработчик
    let handler = |candle: Candle| {
        get_logger().info(
            LogComponent::Infrastructure("BinanceWS"),
            &format!("✅ Test received: ${:.2}", candle.ohlcv.close.value())
        );
    };
    
    // Запускаем на 10 секунд для теста
    if let Err(e) = client.start_stream(handler).await {
        get_logger().error(
            LogComponent::Infrastructure("BinanceWS"),
            &format!("❌ Stream error: {}", e)
        );
        return Err(JsValue::from_str(&e));
    }
    
    get_logger().info(
        LogComponent::Infrastructure("BinanceWS"),
        "✅ Binance WebSocket test completed"
    );
    Ok(())
} 