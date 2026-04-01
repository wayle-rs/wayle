### Configuration Wayle - Module Meteo

## Configuration du module Meteo

settings-modules-weather-provider = Fournisseur
    .description = Fournisseur de donnees meteo

settings-modules-weather-location = Emplacement
    .description = Nom de ville ou coordonnees "lat,lon"

settings-modules-weather-units = Unite temperature
    .description = Metrique (Celsius) ou imperial (Fahrenheit)

settings-modules-weather-format = Format du libellé
    .description = Chaine de format avec placeholders: {"{{ temp }}"}, {"{{ temp_unit }}"}, {"{{ feels_like }}"}, {"{{ condition }}"}, {"{{ humidity }}"}, {"{{ wind_speed }}"}, {"{{ wind_dir }}"}, {"{{ high }}"}, {"{{ low }}"}

settings-modules-weather-refresh-interval = Intervalle d'actualisation
    .description = Intervalle de sondage en secondes

settings-modules-weather-visual-crossing-key = Visual Crossing Key
    .description = Cle API pour Visual Crossing. Utilisez $VAR_NAME pour referencer variables .env

settings-modules-weather-weatherapi-key = WeatherAPI Key
    .description = Cle API pour WeatherAPI.com. Utilisez $VAR_NAME pour referencer variables .env

settings-modules-weather-icon-name = Icone secours
    .description = Icone affichee quand donnees meteo indisponibles

settings-modules-weather-border-show = Afficher la bordure
    .description = Afficher une bordure autour du bouton

settings-modules-weather-border-color = Couleur de bordure
    .description = Valeur de couleur de bordure

settings-modules-weather-icon-show = Afficher l'icone
    .description = Afficher l'icone du module

settings-modules-weather-icon-color = Couleur de l'icone
    .description = Couleur de premier plan de l'icone

settings-modules-weather-icon-bg-color = Arriere-plan de l'icone
    .description = Couleur d'arriere-plan du conteneur d'icone

settings-modules-weather-label-show = Afficher le libellé
    .description = Afficher le libellé de temperature

settings-modules-weather-label-color = Couleur du libellé
    .description = Couleur du texte du libellé

settings-modules-weather-label-max-length = Longueur max du libellé
    .description = Nombre maximal de caracteres avant troncature

settings-modules-weather-button-bg-color = Arriere-plan du bouton
    .description = Couleur d'arriere-plan du bouton

settings-modules-weather-right-click = Clic droit
    .description = Commande shell au clic droit

settings-modules-weather-middle-click = Clic milieu
    .description = Commande shell au clic milieu

settings-modules-weather-scroll-up = Defilement vers le haut
    .description = Commande shell au defilement vers le haut

settings-modules-weather-scroll-down = Defilement vers le bas
    .description = Commande shell au defilement vers le bas
