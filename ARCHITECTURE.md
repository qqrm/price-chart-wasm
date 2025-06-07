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

---

## 🔍 Детальная система трекинга Viewport

### Общий принцип

Система трекинга viewport обеспечивает:
- **Детальное отслеживание изменений** - анализ каждого типа изменения viewport отдельно
- **Условное обновление** - uniform буфер обновляется только при реальных изменениях
- **Селективные обновления** - обновляются только те части uniform буфера, которые изменились
- **Статистика оптимизации** - подробная статистика для мониторинга производительности

### Структуры данных

#### ViewportState
```rust
#[derive(Debug, Clone, PartialEq)]
struct ViewportState {
    width: u32,           // Ширина области просмотра
    height: u32,          // Высота области просмотра
    min_price: f32,       // Минимальная цена в области просмотра
    max_price: f32,       // Максимальная цена в области просмотра
    start_time: f64,      // Начальное время области просмотра
    end_time: f64,        // Конечное время области просмотра
    candle_count: usize,  // Количество свечей в данных
}
```

#### ViewportChangeType
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ViewportChangeType {
    SizeChange { 
        old_size: (u32, u32), 
        new_size: (u32, u32) 
    },
    PriceRangeChange { 
        old_range: (f32, f32), 
        new_range: (f32, f32) 
    },
    TimeRangeChange { 
        old_range: (f64, f64), 
        new_range: (f64, f64) 
    },
    CandleCountChange { 
        old_count: usize, 
        new_count: usize 
    },
    MultipleChanges(Vec<ViewportChangeType>),
}
```

#### ViewportChangeStats
```rust
#[derive(Debug, Clone)]
pub struct ViewportChangeStats {
    pub size_changes: u32,          // Количество изменений размера
    pub price_range_changes: u32,   // Количество изменений ценового диапазона
    pub time_range_changes: u32,    // Количество изменений временного диапазона
    pub candle_count_changes: u32,  // Количество изменений данных
    pub total_viewport_changes: u32, // Общее количество изменений viewport
    pub last_change_type: Option<ViewportChangeType>, // Последний тип изменения
}
```

### Алгоритм работы

#### 1. Анализ изменений
```rust
fn analyze_viewport_changes(&self, new_viewport: &ViewportState) -> Vec<ViewportChangeType>
```
- Сравнивает каждое поле нового состояния с кэшированным
- Использует `f32::EPSILON` и `f64::EPSILON` для точного сравнения floating-point значений
- Возвращает вектор всех обнаруженных изменений

#### 2. Селективное обновление uniform буфера
```rust
fn update_uniforms_from_chart_selective(
    &mut self, 
    chart: &Chart, 
    queue: &Queue, 
    changes: &[ViewportChangeType]
)
```
**Оптимизация:** 
- Обновляет только те части uniform буфера, которые действительно изменились
- Избегает ненужных записей в GPU память
- Поддерживает рекурсивную обработку множественных изменений

**Селективные обновления:**
- `SizeChange`: обновляет только `uniforms.viewport[0..2]` (ширина/высота)
- `PriceRangeChange`: обновляет только `uniforms.viewport[2..4]` (min/max цена)
- `TimeRangeChange`: обновляет только `uniforms.time_range`
- `CandleCountChange`: логирует изменение (не требует uniform обновления)

#### 3. Статистика и мониторинг
```rust
fn update_viewport_change_stats(&mut self, changes: &[ViewportChangeType])
```
- Увеличивает счетчики для каждого типа изменения
- Сохраняет информацию о последнем изменении
- Предоставляет данные для анализа производительности

### Преимущества системы

#### Производительность
- ⚡ **Минимальные GPU операции**: обновление только при реальных изменениях
- 🔄 **Селективные обновления**: обновление только измененных частей uniform буфера
- 📊 **Детальная аналитика**: мониторинг эффективности оптимизаций

#### Надежность
- 🎯 **Точное сравнение**: использование EPSILON для floating-point сравнений
- 📝 **Подробное логирование**: детальная информация о каждом типе изменения
- 🔍 **Отладка**: легко отследить причины обновлений

#### Масштабируемость
- 📈 **Статистика**: счетчики для каждого типа изменения
- 🔧 **Расширяемость**: легко добавить новые типы изменений
- 📋 **Мониторинг**: получение детальной статистики для анализа

### Использование

```rust
// Основной цикл обновления
pub fn update_from_chart(&mut self, chart: &Chart, device: &Device, queue: &Queue) {
    let current_viewport = self.extract_viewport_state(chart);
    let viewport_changes = self.analyze_viewport_changes(&current_viewport);
    
    // Условное обновление только при изменениях
    if !viewport_changes.is_empty() {
        self.update_uniforms_from_chart_selective(chart, queue, &viewport_changes);
        self.update_viewport_change_stats(&viewport_changes);
        // ... дополнительная логика
    }
}

// Получение статистики
let stats = renderer.get_viewport_change_stats();
println!("Size changes: {}, Price changes: {}", 
    stats.size_changes, stats.price_range_changes);
```

### Логирование

Система предоставляет детальное логирование:
- 📐 **Size changes**: `Size changed: 800x600`
- 💰 **Price range changes**: `Price range changed: 50000.00 - 52000.00`
- ⏰ **Time range changes**: `Time range changed: 1640995200 - 1640995800 (range: 600)`
- 🕯️ **Candle count changes**: `Candle count changed: 150 candles`
- ✅ **GPU updates**: `Uniform buffer updated on GPU`

### Интеграция с double buffering

Система совместима с механизмом double buffering:
```rust
pub fn update_with_double_buffering(&mut self, chart: &Chart, device: &Device, queue: &Queue) {
    let viewport_changes = self.analyze_viewport_changes(&current_viewport);
    
    if !viewport_changes.is_empty() {
        self.update_uniforms_from_chart_selective(chart, queue, &viewport_changes);
        // Подготовка следующего буфера при необходимости
    }
}
```

Это обеспечивает оптимальную производительность при высокой частоте обновлений (60 FPS). 