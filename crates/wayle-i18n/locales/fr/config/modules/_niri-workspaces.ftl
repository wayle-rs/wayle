### Configuration Wayle - Module Espaces de travail Niri

## Configuration du module Espaces de travail Niri

settings-modules-niri-workspaces-min-workspace-count = Espaces de travail minimum
    .description = Toujours afficher les espaces de travail numérotés jusqu'à cet indice, même vides

settings-modules-niri-workspaces-monitor-specific = Spécifique au moniteur
    .description = Afficher uniquement les espaces de travail de ce moniteur

settings-modules-niri-workspaces-hide-trailing-empty = Masquer le vide final
    .description = Masquer l'espace de travail vide alloué automatiquement par Niri en fin de sortie

settings-modules-niri-workspaces-display-mode = Mode d'affichage
    .description = Ce qui identifie chaque espace de travail (libellé, icône ou rien)

settings-modules-niri-workspaces-label-strategy = Stratégie de libellé
    .description = Comment composer le libellé de l'espace de travail à partir du nom et de l'indice

settings-modules-niri-workspaces-divider = Séparateur
    .description = Texte entre l'identifiant de l'espace de travail et les icônes d'applications

settings-modules-niri-workspaces-urgent-show = Afficher l'urgence
    .description = Animation pulsée sur les espaces de travail avec des fenêtres urgentes

settings-modules-niri-workspaces-urgent-mode = Mode d'urgence
    .description = Mettre en évidence tout l'espace de travail ou seulement l'icône urgente

settings-modules-niri-workspaces-app-icons-show = Afficher les icônes d'applications
    .description = Afficher les icônes de fenêtres par espace de travail

settings-modules-niri-workspaces-app-icons-dedupe = Dédupliquer les icônes
    .description = Afficher une seule icône par app_id plutôt qu'une par fenêtre

settings-modules-niri-workspaces-app-icons-fallback = Icône de repli
    .description = Icône pour les fenêtres non reconnues par la carte d'icônes

settings-modules-niri-workspaces-app-icons-empty = Icône vide
    .description = Icône affichée quand un espace de travail n'a aucune fenêtre

settings-modules-niri-workspaces-icon-gap = Écart entre icônes
    .description = Espacement entre les icônes d'applications

settings-modules-niri-workspaces-workspace-padding = Marge intérieure de l'espace de travail
    .description = Marge intérieure dans la direction de la barre

settings-modules-niri-workspaces-icon-size = Taille des icônes
    .description = Multiplicateur d'échelle pour les icônes d'espaces de travail (0.25-3.0)

settings-modules-niri-workspaces-label-size = Taille des libellés
    .description = Multiplicateur d'échelle pour les libellés d'espaces de travail (0.25-3.0)

settings-modules-niri-workspaces-workspace-ignore = Ignorer les espaces de travail
    .description = Motifs glob correspondant au nom, à l'indice ou à l'identifiant des espaces de travail à masquer

settings-modules-niri-workspaces-active-indicator = Indicateur actif
    .description = Style visuel pour l'espace de travail actif

settings-modules-niri-workspaces-active-color = Couleur active
    .description = Couleur des icônes et libellés de l'espace de travail actif

settings-modules-niri-workspaces-occupied-color = Couleur occupé
    .description = Couleur des icônes et libellés des espaces de travail occupés

settings-modules-niri-workspaces-empty-color = Couleur vide
    .description = Couleur des icônes et libellés des espaces de travail vides

settings-modules-niri-workspaces-container-bg-color = Arrière-plan du conteneur
    .description = Couleur d'arrière-plan du conteneur d'espaces de travail

settings-modules-niri-workspaces-border-show = Afficher la bordure
    .description = Afficher une bordure autour du conteneur d'espaces de travail

settings-modules-niri-workspaces-border-color = Couleur de la bordure
    .description = Couleur de la bordure du conteneur d'espaces de travail

settings-modules-niri-workspaces-workspace-map = Carte des espaces de travail
    .description = Remplacements d'icônes et de couleurs par espace de travail, par nom ou identifiant

settings-modules-niri-workspaces-app-icon-map = Carte des icônes d'applications
    .description = Correspondances app_id ou titre de fenêtre vers des icônes

settings-modules-niri-workspaces-left-click = Clic gauche
    .description = Action au clic gauche

settings-modules-niri-workspaces-middle-click = Clic central
    .description = Action au clic central

settings-modules-niri-workspaces-right-click = Clic droit
    .description = Action au clic droit

settings-modules-niri-workspaces-scroll-up = Défilement vers le haut
    .description = Action au défilement vers le haut

settings-modules-niri-workspaces-scroll-down = Défilement vers le bas
    .description = Action au défilement vers le bas


## Variantes de LabelStrategy
enum-label-strategy-index = Indice
enum-label-strategy-name-or-index = Nom ou indice
enum-label-strategy-name-only = Nom seulement
enum-label-strategy-index-and-name = Indice et nom
