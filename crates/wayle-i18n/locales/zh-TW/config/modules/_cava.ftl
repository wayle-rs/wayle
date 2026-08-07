### Wayle Configuration - Cava Module

## Audio Processing

settings-modules-cava-bars = 長條數量
    .description = 頻率長條數量

settings-modules-cava-framerate = 畫面更新率
    .description = 視覺化圖表更新率（每秒畫格數）

settings-modules-cava-stereo = 立體聲
    .description = 將長條分配至左、右音訊聲道

settings-modules-cava-noise-reduction = 降噪
    .description = 降噪濾鏡強度（0.0 快速／有噪音，1.0 緩慢／平滑）

settings-modules-cava-monstercat = Monstercat 平滑化
    .description = 相鄰長條之間的平滑衰減（0.0 表示關閉）

settings-modules-cava-waves = 波形平滑化
    .description = 波形樣式的平滑化次數（0 表示關閉）

settings-modules-cava-low-cutoff = 低頻截止
    .description = 低頻截止（Hz）

settings-modules-cava-high-cutoff = 高頻截止
    .description = 高頻截止（Hz）

settings-modules-cava-input = 輸入方式
    .description = 音訊擷取後端

settings-modules-cava-source = 來源
    .description = 音訊來源識別碼（使用 "auto" 自動選擇）

## Visualization

settings-modules-cava-style = 樣式
    .description = 視覺化圖表的呈現樣式（長條、波形或峰值）

settings-modules-cava-direction = 方向
    .description = 長條相對於列邊緣的成長方向

settings-modules-cava-color = 顏色
    .description = 視覺化圖表顏色

settings-modules-cava-bar-width = 長條寬度
    .description = 每個頻率長條的寬度（像素）

settings-modules-cava-bar-gap = 長條間距
    .description = 頻率長條之間的間距（像素）

## Container

settings-modules-cava-internal-padding = 內部內距
    .description = 視覺化圖表兩端的內距（rem）

settings-modules-cava-button-bg-color = 背景顏色
    .description = 模組背景顏色

settings-modules-cava-border-show = 顯示邊框
    .description = 顯示視覺化圖表周圍的邊框

settings-modules-cava-border-color = 邊框顏色
    .description = 邊框顏色代碼

settings-modules-cava-left-click = 左鍵點擊
    .description = 左鍵點擊動作

settings-modules-cava-right-click = 右鍵點擊
    .description = 右鍵點擊動作

settings-modules-cava-middle-click = 中鍵點擊
    .description = 中鍵點擊動作

settings-modules-cava-scroll-up = 向上滾動
    .description = 向上滾動動作

settings-modules-cava-scroll-down = 向下滾動
    .description = 向下滾動動作


## CavaStyle variants
enum-cava-style-bars = 長條
enum-cava-style-wave = 波形
enum-cava-style-peaks = 峰值

## CavaDirection variants
enum-cava-direction-normal = 一般
enum-cava-direction-reverse = 反向
enum-cava-direction-mirror = 鏡像
