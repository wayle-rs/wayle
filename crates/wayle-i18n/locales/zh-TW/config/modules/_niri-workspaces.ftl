### Wayle Configuration - Niri Workspaces Module

## Niri Workspaces Module Configuration

settings-modules-niri-workspaces-min-workspace-count = 最少工作區數量
    .description = 即使沒有內容，也一律顯示到此索引為止的數字命名工作區

settings-modules-niri-workspaces-monitor-specific = 指定螢幕
    .description = 僅顯示此螢幕上的工作區

settings-modules-niri-workspaces-hide-trailing-empty = 隱藏尾端空白
    .description = 隱藏 Niri 在每個輸出末端自動配置的空白工作區

settings-modules-niri-workspaces-display-mode = 顯示模式
    .description = 每個工作區的識別方式（標籤、圖示或無）

settings-modules-niri-workspaces-label-strategy = 標籤策略
    .description = 如何由名稱與索引組合工作區標籤

settings-modules-niri-workspaces-divider = 分隔符號
    .description = 工作區識別與應用程式圖示之間的文字

settings-modules-niri-workspaces-urgent-show = 顯示緊急狀態
    .description = 有緊急視窗的工作區播放脈動動畫

settings-modules-niri-workspaces-urgent-mode = 緊急模式
    .description = 脈動整個工作區或僅脈動緊急應用程式圖示

settings-modules-niri-workspaces-app-icons-show = 顯示應用程式圖示
    .description = 每個工作區顯示視窗圖示

settings-modules-niri-workspaces-app-icons-dedupe = 合併重複圖示
    .description = 每個 app_id 顯示一個圖示，而不是每個視窗一個

settings-modules-niri-workspaces-app-icons-fallback = 備用圖示
    .description = 未被 app-icon-map 對應的視窗所使用的圖示

settings-modules-niri-workspaces-app-icons-empty = 空白圖示
    .description = 工作區沒有應用程式視窗時顯示的圖示

settings-modules-niri-workspaces-icon-gap = 圖示間距
    .description = 應用程式圖示之間的間距

settings-modules-niri-workspaces-workspace-padding = 工作區內距
    .description = 沿列方向的內距

settings-modules-niri-workspaces-icon-size = 圖示大小
    .description = 工作區圖示的縮放倍率（0.25–3.0）

settings-modules-niri-workspaces-label-size = 標籤大小
    .description = 工作區標籤的縮放倍率（0.25–3.0）

settings-modules-niri-workspaces-workspace-ignore = 忽略工作區
    .description = 用於比對名稱、索引或 ID 的 glob 模式

settings-modules-niri-workspaces-active-indicator = 作用中指示器
    .description = 作用中工作區的視覺樣式

settings-modules-niri-workspaces-active-color = 作用中顏色
    .description = 作用中工作區圖示與標籤的顏色

settings-modules-niri-workspaces-occupied-color = 已使用顏色
    .description = 已使用工作區圖示與標籤的顏色

settings-modules-niri-workspaces-empty-color = 空白顏色
    .description = 空白工作區圖示與標籤的顏色

settings-modules-niri-workspaces-container-bg-color = 容器背景
    .description = 工作區容器的背景顏色

settings-modules-niri-workspaces-border-show = 顯示邊框
    .description = 顯示工作區容器周圍的邊框

settings-modules-niri-workspaces-border-color = 邊框顏色
    .description = 工作區容器的邊框顏色

settings-modules-niri-workspaces-workspace-map = 工作區對照表
    .description = 依名稱或 ID 指定的每個工作區圖示與顏色覆寫

settings-modules-niri-workspaces-app-icon-map = 應用程式圖示對照表
    .description = 視窗 app_id 或標題到圖示的對照表

settings-modules-niri-workspaces-left-click = 左鍵點擊
    .description = 左鍵點擊動作

settings-modules-niri-workspaces-middle-click = 中鍵點擊
    .description = 中鍵點擊動作

settings-modules-niri-workspaces-right-click = 右鍵點擊
    .description = 右鍵點擊動作

settings-modules-niri-workspaces-scroll-up = 向上滾動
    .description = 向上滾動動作

settings-modules-niri-workspaces-scroll-down = 向下滾動
    .description = 向下滾動動作


## LabelStrategy variants
enum-label-strategy-index = 索引
enum-label-strategy-name-or-index = 名稱或索引
enum-label-strategy-name-only = 僅名稱
enum-label-strategy-index-and-name = 索引與名稱
