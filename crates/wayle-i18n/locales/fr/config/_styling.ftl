### Configuration Wayle - Paramètres de style

## Configuration du style

settings-styling-scale = Échelle
    .description = Multiplicateur d'échelle pour les menus déroulants, les fenêtres contextuelles et les dialogues

settings-styling-rounding = Arrondis
    .description = Arrondi des coins pour les menus déroulants, les fenêtres contextuelles et les dialogues

settings-styling-theme-provider = Fournisseur de thème
    .description = Source de la palette de couleurs (wayle, matugen, pywal, wallust)

settings-styling-theming-monitor = Moniteur de thème
    .description = Moniteur dont le fond d'écran pilote l'extraction de couleurs

## Configuration Matugen

settings-styling-matugen-scheme = Schéma Matugen
    .description = Type de schéma de couleurs pour la génération de palette Matugen

settings-styling-matugen-contrast = Contraste Matugen
    .description = Niveau de contraste pour Matugen (-1.0 à 1.0)

settings-styling-matugen-source-color = Couleur source Matugen
    .description = Indice de couleur source pour la génération de palette Matugen

settings-styling-matugen-light = Mode clair Matugen
    .description = Générer un schéma de couleurs clair avec Matugen

## Configuration Wallust

settings-styling-wallust-palette = Palette Wallust
    .description = Mode de palette pour la génération de couleurs Wallust

settings-styling-wallust-saturation = Saturation Wallust
    .description = Remplacement de la saturation des couleurs pour Wallust (0 pour désactiver)

settings-styling-wallust-check-contrast = Vérification du contraste Wallust
    .description = Indique si Wallust impose un contraste minimal entre les couleurs

settings-styling-wallust-backend = Moteur Wallust
    .description = Méthode d'échantillonnage d'image pour l'extraction de couleurs Wallust

settings-styling-wallust-colorspace = Espace colorimétrique Wallust
    .description = Espace colorimétrique utilisé pour la sélection de couleur dominante dans Wallust

settings-styling-wallust-apply-globally = Application globale Wallust
    .description = Appliquer les couleurs Wallust aux terminaux et aux outils externes

## Configuration Pywal

settings-styling-pywal-saturation = Saturation Pywal
    .description = Saturation des couleurs pour Pywal (0.0 à 1.0)

settings-styling-pywal-contrast = Contraste Pywal
    .description = Rapport de contraste minimal pour Pywal (1.0 à 21.0)

settings-styling-pywal-light = Mode clair Pywal
    .description = Générer un schéma de couleurs clair avec Pywal

settings-styling-pywal-apply-globally = Application globale Pywal
    .description = Appliquer les couleurs Pywal aux terminaux et aux outils externes

## Configuration de la palette

settings-palette-bg = Arrière-plan
    .description = Couleur d'arrière-plan de base (couche la plus sombre)

settings-palette-surface = Surface
    .description = Couleur d'arrière-plan des cartes et de la barre latérale

settings-palette-elevated = Élevé
    .description = Couleur d'arrière-plan des éléments en relief

settings-palette-fg = Premier plan
    .description = Couleur du texte principal

settings-palette-fg-muted = Premier plan atténué
    .description = Couleur du texte secondaire

settings-palette-primary = Primaire
    .description = Couleur d'accentuation pour les éléments interactifs

settings-palette-red = Rouge
    .description = Couleur sémantique rouge

settings-palette-yellow = Jaune
    .description = Couleur sémantique jaune

settings-palette-green = Vert
    .description = Couleur sémantique verte

settings-palette-blue = Bleu
    .description = Couleur sémantique bleue


## Variantes de ThemeProvider
enum-theme-provider-wayle = Wayle
enum-theme-provider-matugen = Matugen
enum-theme-provider-pywal = Pywal
enum-theme-provider-wallust = Wallust

## Variantes de MatugenScheme
enum-matugen-scheme-content = Contenu
enum-matugen-scheme-expressive = Expressif
enum-matugen-scheme-fidelity = Fidélité
enum-matugen-scheme-fruit-salad = Salade de fruits
enum-matugen-scheme-monochrome = Monochrome
enum-matugen-scheme-neutral = Neutre
enum-matugen-scheme-rainbow = Arc-en-ciel
enum-matugen-scheme-tonal-spot = Tache tonale
enum-matugen-scheme-vibrant = Vibrant

## Variantes de WallustPalette
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

## Variantes de WallustBackend
enum-wallust-backend-full = Complet
enum-wallust-backend-resized = Redimensionné
enum-wallust-backend-wal = Wal
enum-wallust-backend-thumb = Miniature
enum-wallust-backend-fastresize = Redimensionnement rapide
enum-wallust-backend-kmeans = K-Means

## Variantes de WallustColorspace
enum-wallust-colorspace-lab = Lab
enum-wallust-colorspace-labmixed = Labmixed
enum-wallust-colorspace-lch = Lch
enum-wallust-colorspace-lchmixed = Lchmixed
enum-wallust-colorspace-lchansi = Lchansi

## Variantes de FontWeightClass
enum-font-weight-class-normal = Normal
enum-font-weight-class-medium = Moyen
enum-font-weight-class-semibold = Semi-gras
enum-font-weight-class-bold = Gras

## Variantes de RoundingLevel
enum-rounding-level-none = Aucun
enum-rounding-level-sm = Petit
enum-rounding-level-md = Moyen
enum-rounding-level-lg = Grand
enum-rounding-level-full = Complet
