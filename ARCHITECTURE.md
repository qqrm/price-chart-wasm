# 🏗️ DDD Architecture - Price Chart WASM

## 📋 Общая структура

```
src/
├── domain/                 # 🏛️ ДОМЕННЫЙ СЛОЙ
│   ├── market_data/       # Агрегат: Рыночные данные
│   │   ├── entities.rs    # Сущности (Candle, CandleSeries)
│   │   ├── value_objects.rs # Value Objects (Price, Volume, OHLCV)
│   │   ├── repositories.rs # Интерфейсы репозиториев
│   │   └── services.rs    # Доменные сервисы (анализ, валидация)
│   └── chart/             # Агрегат: Графики
│       ├── entities.rs    # Сущности (Chart, Indicator, RenderLayer)
│       ├── value_objects.rs # Value Objects (Viewport, Color, ChartType)
│       └── services.rs    # Сервисы рендеринга
├── infrastructure/        # 🔧 ИНФРАСТРУКТУРНЫЙ СЛОЙ
│   ├── websocket/         # WebSocket реализации
│   │   ├── dto.rs        # DTO для внешних API
│   │   └── binance_client.rs # Binance WebSocket клиент
│   └── rendering/         # WebGPU рендеринг
├── application/           # 🎯 СЛОЙ ПРИЛОЖЕНИЯ
│   ├── use_cases.rs      # Use Cases и координаторы
│   └── chart_service.rs  # Сервисы приложения
└── presentation/          # 🌐 ПРЕЗЕНТАЦИОННЫЙ СЛОЙ
    ├── wasm_api.rs       # WASM API для JavaScript
    └── mod.rs            # Экспорты
```

## 🏛️ Domain Layer - Ядро системы

**Принципы:**
- ✅ Никаких внешних зависимостей
- ✅ Только бизнес-логика и валидация  
- ✅ Независимые, тестируемые модели
- ✅ Инварианты и доменные правила

### Сущности (Entities)
```rust
// Основная доменная сущность
pub struct Candle {
    pub timestamp: Timestamp,
    pub ohlcv: OHLCV,
}

// Агрегат для управления коллекцией свечей
pub struct CandleSeries {
    candles: Vec<Candle>,
    max_size: usize,
}
```

### Value Objects
```rust
// Неизменяемые объекты-значения
pub struct Price(f32);
pub struct Volume(f32);
pub struct Timestamp(u64);
pub struct Symbol(String);
```

### Доменные сервисы
```rust
// Сервисы для бизнес-логики, которая не принадлежит сущностям
pub struct MarketAnalysisService;  // SMA, волатильность, экстремумы
pub struct DataValidationService;  // Валидация свечей и последовательностей
```

## 🔧 Infrastructure Layer - Технические детали

**Принципы:**
- ✅ Реализует интерфейсы из domain
- ✅ Содержит технические детали (WebSocket, WebGPU)
- ✅ Легко подменяется для тестирования
- ✅ DTO для преобразования внешних данных

### WebSocket Infrastructure
```rust
// DTO для Binance API
pub struct BinanceKlineData { ... }

// Реализация репозитория
impl MarketDataRepository for BinanceWebSocketClient {
    fn subscribe_to_updates(...) -> Result<(), JsValue> { ... }
}
```

## 🎯 Application Layer - Координация

**Принципы:**
- ✅ Use Cases для бизнес-сценариев
- ✅ Координация между domain и infrastructure
- ✅ Управление транзакциями и состоянием
- ✅ Минимальная логика - только оркестрация

### Use Cases
```rust
// Подключение к данным
pub struct ConnectToMarketDataUseCase<T: MarketDataRepository> { ... }

// Анализ рынка  
pub struct AnalyzeMarketDataUseCase { ... }

// Рендеринг графика
pub struct RenderChartUseCase { ... }

// Координатор всех сценариев
pub struct ChartApplicationCoordinator<T> { ... }
```

## 🌐 Presentation Layer - API для веба

**Принципы:**
- ✅ Минимальная логика - только мост к application
- ✅ WASM API для JavaScript
- ✅ Преобразование типов для веба
- ✅ Обратная совместимость

### WASM API
```rust
#[wasm_bindgen]
pub struct PriceChartApi {
    // Внутреннее состояние скрыто от JS
}

#[wasm_bindgen]
impl PriceChartApi {
    #[wasm_bindgen(js_name = connectToSymbol)]
    pub fn connect_to_symbol(&mut self, symbol: &str, interval: &str) -> Result<(), JsValue> {
        // Делегирует в application слой
    }
}
```

## 🔄 Поток данных

```
JavaScript API
       ↓
🌐 Presentation Layer (WASM API)
       ↓
🎯 Application Layer (Use Cases)
       ↓
🏛️ Domain Layer (Entities, Services)
       ↓
🔧 Infrastructure Layer (WebSocket, WebGPU)
       ↓
External APIs (Binance, Browser)
```

## ✅ Преимущества архитектуры

1. **Тестируемость** - Domain слой тестируется изолированно
2. **Независимость** - Бизнес-логика не зависит от технологий
3. **Подменяемость** - Infrastructure легко заменить (mock, другая биржа)
4. **Читаемость** - Четкое разделение ответственности
5. **Расширяемость** - Новые фичи добавляются в правильные слои

## 🚀 Дальнейшее развитие

- [ ] Добавить больше бирж в infrastructure
- [ ] Расширить доменные сервисы (больше индикаторов)
- [ ] Улучшить координаторы в application
- [ ] Добавить события (Event Sourcing)
- [ ] Реализовать CQRS паттерн 