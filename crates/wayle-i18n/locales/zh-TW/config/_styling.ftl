### Wayle Configuration - Styling Settings

## Styling Configuration

settings-styling-scale = 縮放
    .description = 下拉選單、彈出面板與對話框的縮放倍率

settings-styling-rounding = 圓角
    .description = 下拉選單、彈出面板與對話框的圓角

settings-styling-theme-provider = 主題提供者
    .description = 色彩調色盤的來源（wayle、matugen、pywal、wallust）

settings-styling-theming-monitor = 主題螢幕
    .description = 用於擷取色彩的桌布所屬螢幕

## Matugen Configuration

settings-styling-matugen-scheme = Matugen 配色方案
    .description = Matugen 調色盤產生使用的配色方案類型

settings-styling-matugen-contrast = Matugen 對比度
    .description = Matugen 的對比程度（-1.0 至 1.0）

settings-styling-matugen-source-color = Matugen 來源色彩
    .description = Matugen 調色盤產生使用的來源色彩索引

settings-styling-matugen-light = Matugen 淺色模式
    .description = 使用 Matugen 產生淺色配色方案

## Wallust Configuration

settings-styling-wallust-palette = Wallust 調色盤
    .description = Wallust 產生色彩時使用的調色盤模式

settings-styling-wallust-saturation = Wallust 飽和度
    .description = Wallust 的色彩飽和度覆寫值（0 表示停用）

settings-styling-wallust-check-contrast = Wallust 檢查對比度
    .description = Wallust 是否強制色彩之間的最低對比度

settings-styling-wallust-backend = Wallust 後端
    .description = Wallust 擷取色彩時使用的圖片取樣方式

settings-styling-wallust-colorspace = Wallust 色彩空間
    .description = Wallust 選取主色時使用的色彩空間

settings-styling-wallust-apply-globally = Wallust 全域套用
    .description = 將 Wallust 色彩套用至終端機與外部工具

## Pywal Configuration

settings-styling-pywal-saturation = Pywal 飽和度
    .description = Pywal 的色彩飽和度（0.0 至 1.0）

settings-styling-pywal-contrast = Pywal 對比度
    .description = Pywal 的最低對比度比例（1.0 至 21.0）

settings-styling-pywal-light = Pywal 淺色模式
    .description = 使用 Pywal 產生淺色配色方案

settings-styling-pywal-apply-globally = Pywal 全域套用
    .description = 將 Pywal 色彩套用至終端機與外部工具

## Palette Configuration

settings-palette-bg = 背景
    .description = 基本背景色（最暗層）

settings-palette-surface = 表面
    .description = 卡片與側邊欄的背景色

settings-palette-elevated = 浮起
    .description = 浮起元素的背景色

settings-palette-fg = 前景
    .description = 主要文字顏色

settings-palette-fg-muted = 弱化前景
    .description = 次要文字顏色

settings-palette-primary = 主要色
    .description = 互動元素的強調色

settings-palette-red = 紅色
    .description = 紅色語意色

settings-palette-yellow = 黃色
    .description = 黃色語意色

settings-palette-green = 綠色
    .description = 綠色語意色

settings-palette-blue = 藍色
    .description = 藍色語意色


## ThemeProvider variants
enum-theme-provider-wayle = Wayle
enum-theme-provider-matugen = Matugen
enum-theme-provider-pywal = Pywal
enum-theme-provider-wallust = Wallust

## MatugenScheme variants
enum-matugen-scheme-content = 內容
enum-matugen-scheme-expressive = 表現
enum-matugen-scheme-fidelity = 保真
enum-matugen-scheme-fruit-salad = 水果沙拉
enum-matugen-scheme-monochrome = 單色
enum-matugen-scheme-neutral = 中性
enum-matugen-scheme-rainbow = 彩虹
enum-matugen-scheme-tonal-spot = 色調焦點
enum-matugen-scheme-vibrant = 鮮豔

## WallustPalette variants
enum-wallust-palette-dark16 = Dark 16
enum-wallust-palette-dark = Dark
enum-wallust-palette-darkcomp = Darkcomp
enum-wallust-palette-darkcomp16 = Darkcomp16
enum-wallust-palette-harddark = Harddark
enum-wallust-palette-harddark16 = Harddark16
enum-wallust-palette-harddarkcomp = Harddarkcomp
enum-wallust-palette-harddarkcomp16 = Harddarkcomp16
enum-wallust-palette-light = Light
enum-wallust-palette-light16 = Light16
enum-wallust-palette-lightcomp = Lightcomp
enum-wallust-palette-lightcomp16 = Lightcomp16
enum-wallust-palette-softdark = Softdark
enum-wallust-palette-softdark16 = Softdark16
enum-wallust-palette-softdarkcomp = Softdarkcomp
enum-wallust-palette-softdarkcomp16 = Softdarkcomp16
enum-wallust-palette-softlight = Softlight
enum-wallust-palette-softlight16 = Softlight16
enum-wallust-palette-softlightcomp = Softlightcomp
enum-wallust-palette-softlightcomp16 = Softlightcomp16
enum-wallust-palette-ansidark = Ansidark
enum-wallust-palette-ansidark16 = Ansidark16

## WallustBackend variants
enum-wallust-backend-full = Full
enum-wallust-backend-resized = Resized
enum-wallust-backend-wal = Wal
enum-wallust-backend-thumb = Thumb
enum-wallust-backend-fastresize = Fastresize
enum-wallust-backend-kmeans = K-Means

## WallustColorspace variants
enum-wallust-colorspace-lab = Lab
enum-wallust-colorspace-labmixed = Labmixed
enum-wallust-colorspace-lch = Lch
enum-wallust-colorspace-lchmixed = Lchmixed
enum-wallust-colorspace-lchansi = Lchansi

## FontWeightClass variants
enum-font-weight-class-normal = 一般
enum-font-weight-class-medium = 中等
enum-font-weight-class-semibold = 半粗體
enum-font-weight-class-bold = 粗體

## RoundingLevel variants
enum-rounding-level-none = 無
enum-rounding-level-sm = 小
enum-rounding-level-md = 中
enum-rounding-level-lg = 大
enum-rounding-level-full = 完整
