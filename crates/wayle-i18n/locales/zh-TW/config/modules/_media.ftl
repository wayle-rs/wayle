### Wayle Configuration - Media Module

## Media Module Configuration

settings-modules-media-icon-type = 圖示類型
    .description = 圖示顯示模式（預設、應用程式、旋轉唱片、應用程式對應）。旋轉唱片模式因動畫會稍微增加 CPU 使用量

settings-modules-media-player-icons = 播放器圖示
    .description = 自訂播放器到圖示的對照表（glob 模式對應圖示名稱）

settings-modules-media-players-ignored = 忽略的播放器
    .description = 從探索中排除的播放器匯流排名稱模式

settings-modules-media-player-priority = 播放器優先順序
    .description = 以符合匯流排名稱的 glob 模式指定播放器優先順序

settings-modules-media-format = 格式
    .description = 含佔位符的標籤格式：{"{{ title }}"}、{"{{ artist }}"}、{"{{ album }}"}、{"{{ status }}"}、{"{{ status_icon }}"}

settings-modules-media-icon-name = 圖示名稱
    .description = 預設模式使用的符號圖示名稱

settings-modules-media-spinning-disc-icon = 旋轉唱片圖示
    .description = 旋轉唱片模式顯示的圖示

settings-modules-media-border-show = 顯示邊框
    .description = 顯示按鈕周圍的邊框

settings-modules-media-border-color = 邊框顏色
    .description = 邊框顏色代碼

settings-modules-media-icon-show = 顯示圖示
    .description = 顯示模組圖示

settings-modules-media-icon-color = 圖示顏色
    .description = 圖示前景色

settings-modules-media-icon-bg-color = 圖示背景
    .description = 圖示容器背景色

settings-modules-media-label-show = 顯示標籤
    .description = 顯示文字標籤

settings-modules-media-label-color = 標籤顏色
    .description = 標籤文字顏色

settings-modules-media-label-max-length = 標籤最大長度
    .description = 截斷前的最大字元數

settings-modules-media-button-bg-color = 按鈕背景
    .description = 按鈕背景顏色

settings-modules-media-left-click = 左鍵點擊
    .description = 左鍵點擊動作

settings-modules-media-right-click = 右鍵點擊
    .description = 右鍵點擊時執行的 Shell 指令

settings-modules-media-middle-click = 中鍵點擊
    .description = 中鍵點擊時執行的 Shell 指令

settings-modules-media-scroll-up = 向上滾動
    .description = 向上滾動時執行的 Shell 指令

settings-modules-media-scroll-down = 向下滾動
    .description = 向下滾動時執行的 Shell 指令


## MediaIconType variants
enum-media-icon-type-default = 預設
enum-media-icon-type-application = 應用程式
enum-media-icon-type-spinning-disc = 旋轉唱片
enum-media-icon-type-application-mapped = 應用程式對應
