### Wayle Configuration - Styling Settings

## Styling Configuration

settings-styling-scale = Scale
    .description = Scale multiplier for dropdowns, popovers, and dialogs

settings-styling-rounding = Rounding
    .description = Corner rounding for dropdowns, popovers, and dialogs

settings-styling-theme-provider = Theme Provider
    .description = Source for color palette (wayle, matugen, pywal, wallust)

settings-styling-theming-monitor = Theming Monitor
    .description = Monitor whose wallpaper drives color extraction

## Matugen Configuration

settings-styling-matugen-scheme = Matugen Scheme
    .description = Color scheme type for matugen palette generation

settings-styling-matugen-contrast = Matugen Contrast
    .description = Contrast level for matugen (-1.0 to 1.0)

settings-styling-matugen-source-color = Matugen Source Color
    .description = Source color index for matugen palette generation

settings-styling-matugen-light = Matugen Light Mode
    .description = Generate a light color scheme with matugen

## Wallust Configuration

settings-styling-wallust-palette = Wallust Palette
    .description = Palette mode for wallust color generation

settings-styling-wallust-saturation = Wallust Saturation
    .description = Color saturation override for wallust (0 to disable)

settings-styling-wallust-check-contrast = Wallust Check Contrast
    .description = Whether wallust enforces minimum contrast between colors

settings-styling-wallust-backend = Wallust Backend
    .description = Image sampling method for wallust color extraction

settings-styling-wallust-colorspace = Wallust Colorspace
    .description = Color space used for dominant color selection in wallust

settings-styling-wallust-apply-globally = Wallust Apply Globally
    .description = Apply wallust colors to terminals and external tools

## Pywal Configuration

settings-styling-pywal-saturation = Pywal Saturation
    .description = Color saturation for pywal (0.0 to 1.0)

settings-styling-pywal-contrast = Pywal Contrast
    .description = Minimum contrast ratio for pywal (1.0 to 21.0)

settings-styling-pywal-light = Pywal Light Mode
    .description = Generate a light color scheme with pywal

settings-styling-pywal-apply-globally = Pywal Apply Globally
    .description = Apply pywal colors to terminals and external tools

## Palette Configuration

settings-palette-bg = Background
    .description = Base background color (darkest layer)

settings-palette-surface = Surface
    .description = Card and sidebar background color

settings-palette-elevated = Elevated
    .description = Raised element background color

settings-palette-fg = Foreground
    .description = Primary text color

settings-palette-fg-muted = Muted Foreground
    .description = Secondary text color

settings-palette-primary = Primary
    .description = Accent color for interactive elements

settings-palette-red = Red
    .description = Red semantic color

settings-palette-yellow = Yellow
    .description = Yellow semantic color

settings-palette-green = Green
    .description = Green semantic color

settings-palette-blue = Blue
    .description = Blue semantic color


## ThemeProvider variants
enum-theme-provider-wayle = Wayle
enum-theme-provider-matugen = Matugen
enum-theme-provider-pywal = Pywal
enum-theme-provider-wallust = Wallust

## MatugenScheme variants
enum-matugen-scheme-content = Content
enum-matugen-scheme-expressive = Expressive
enum-matugen-scheme-fidelity = Fidelity
enum-matugen-scheme-fruit-salad = Fruit Salad
enum-matugen-scheme-monochrome = Monochrome
enum-matugen-scheme-neutral = Neutral
enum-matugen-scheme-rainbow = Rainbow
enum-matugen-scheme-tonal-spot = Tonal Spot
enum-matugen-scheme-vibrant = Vibrant

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
enum-font-weight-class-normal = Normal
enum-font-weight-class-medium = Medium
enum-font-weight-class-semibold = Semibold
enum-font-weight-class-bold = Bold

## RoundingLevel variants
enum-rounding-level-none = None
enum-rounding-level-sm = Small
enum-rounding-level-md = Medium
enum-rounding-level-lg = Large
enum-rounding-level-full = Full
