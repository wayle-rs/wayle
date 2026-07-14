### Libellés des modules de la barre

## Réseau
bar-network-connecting = Connexion…
bar-network-disconnected = Déconnecté
bar-network-wired = Filaire
bar-network-wifi-fallback = Wi-Fi
bar-network-no-wifi = Pas de Wi-Fi
bar-network-no-ethernet = Pas d'Ethernet
bar-network-offline = Hors ligne

## Batterie
bar-battery-unavailable = N/D

## Bluetooth
bar-bluetooth-disabled = Désactivé
bar-bluetooth-disconnected = Déconnecté
bar-bluetooth-connected-count = { $count ->
    [one] { $count } connecté
   *[other] { $count } connectés
}

## Titre de la fenêtre
bar-window-title-empty = Bureau

## Inhibition de la veille
bar-idle-inhibit-on = Activé
bar-idle-inhibit-off = Désactivé

## Mode de raccourcis
bar-keybind-mode-default = par défaut

## Hyprsunset
bar-hyprsunset-on = Activé
bar-hyprsunset-off = Désactivé

## Stockage
bar-storage-multiple = Multiple

## Média
bar-media-playing = Lecture
bar-media-paused = En pause
bar-media-stopped = Arrêté

### Menu déroulant des notifications

notification-dropdown-title = Notifications
notification-dropdown-empty-title = Aucune notification
notification-dropdown-empty-description = Aucune nouvelle notification
notification-dropdown-clear-all = Tout effacer
notification-dropdown-dnd-label = Ne pas déranger
notification-dropdown-group-clear = Effacer
notification-dropdown-group-more = { $count } de plus
notification-dropdown-unknown-app = Inconnu
notification-dropdown-time-just-now = À l'instant
notification-dropdown-time-minutes-ago = Il y a { $minutes } min
notification-dropdown-time-hours-ago = Il y a { $hours } h

### Popup de notification

notification-popup-unknown-app = Inconnu
notification-popup-time-just-now = À l'instant
notification-popup-time-minutes-ago = Il y a { $minutes } min
notification-popup-time-hours-ago = Il y a { $hours } h

### Affichage à l'écran

## Libellés des bascules
osd-caps-lock = Verr. maj.
osd-num-lock = Verr. num.
osd-scroll-lock = Arrêt défil.

## État des bascules
# { $label } est le nom de la bascule (p. ex. « Verr. maj. »)
osd-toggle-on = { $label } activé
osd-toggle-off = { $label } désactivé

### Conditions météorologiques

weather-clear = Dégagé
weather-partly-cloudy = Partiellement nuageux
weather-cloudy = Nuageux
weather-overcast = Couvert
weather-mist = Brume
weather-fog = Brouillard
weather-light-rain = Pluie légère
weather-rain = Pluie
weather-heavy-rain = Forte pluie
weather-drizzle = Bruine
weather-light-snow = Neige légère
weather-snow = Neige
weather-heavy-snow = Forte neige
weather-sleet = Grésil
weather-thunderstorm = Orage
weather-windy = Venteux
weather-hail = Grêle
weather-unknown = Inconnu

### Menu déroulant audio

dropdown-audio-title = Audio
dropdown-audio-output = Sortie
dropdown-audio-input = Entrée
dropdown-audio-output-devices = Périphériques de sortie
dropdown-audio-input-devices = Périphériques d'entrée
dropdown-audio-app-volume = Volume des applications
dropdown-audio-no-device = Aucun périphérique trouvé
dropdown-audio-no-devices-title = Aucun périphérique audio
dropdown-audio-no-devices-description = Aucun périphérique de sortie ou d'entrée audio trouvé
dropdown-audio-no-apps = Aucune application ne lit de l'audio
dropdown-audio-settings = Paramètres audio

### Menu déroulant de la batterie

dropdown-battery-title = Batterie

## États principaux
dropdown-battery-on-battery = Sur batterie
dropdown-battery-charging = En charge
dropdown-battery-plugged-in = Branché
dropdown-battery-critical = Critique

## Affichage de la durée
dropdown-battery-duration-hm = { $hours } h { $minutes } min
dropdown-battery-duration-m = { $minutes } min
dropdown-battery-time-remaining = { $duration } restant
dropdown-battery-time-until-full = { $duration } avant la charge complète

## Détails
dropdown-battery-draw = Consommation
dropdown-battery-input = Entrée
dropdown-battery-input-watts = { $watts } en entrée
dropdown-battery-capacity = Capacité
dropdown-battery-charged = Chargé
dropdown-battery-health = Santé

## Limite de charge
dropdown-battery-charge-limit = Limite de charge
dropdown-battery-limit-to = Limiter à { $threshold } %
dropdown-battery-resumes-at = Reprend la charge à { $threshold } %
dropdown-battery-charge-limit-not-supported = Limite de charge non prise en charge sur cet appareil

## Profil d'alimentation
dropdown-battery-power-profile = Profil d'alimentation
dropdown-battery-profile-saver = Économie
dropdown-battery-profile-balanced = Équilibré
dropdown-battery-profile-performance = Performance
dropdown-battery-power-profile-not-available = Le démon de profils d'alimentation doit être actif

## Aucune batterie
dropdown-battery-no-battery-title = Aucune batterie détectée
dropdown-battery-no-battery-description = Alimentation sur secteur

### Menu déroulant Bluetooth

dropdown-bluetooth-title = Bluetooth
dropdown-bluetooth-my-devices = Mes appareils
dropdown-bluetooth-available-devices = Appareils disponibles
dropdown-bluetooth-connected = Connecté
dropdown-bluetooth-scanning = Recherche
dropdown-bluetooth-connect = Connecter
dropdown-bluetooth-disconnect = Déconnecter
dropdown-bluetooth-forget = Oublier
dropdown-bluetooth-pair = Jumeler
dropdown-bluetooth-cancel = Annuler
dropdown-bluetooth-confirm = Confirmer
dropdown-bluetooth-reject = Rejeter
dropdown-bluetooth-try-again = Réessayer
dropdown-bluetooth-allow = Autoriser
dropdown-bluetooth-deny = Refuser

## État de l'appareil
dropdown-bluetooth-battery = { $percent } %
dropdown-bluetooth-paired = Jumelé
dropdown-bluetooth-not-connected = Non connecté
dropdown-bluetooth-new-device = Nouvel appareil
dropdown-bluetooth-status-connecting = Connexion…
dropdown-bluetooth-status-disconnecting = Déconnexion…
dropdown-bluetooth-status-forgetting = Suppression…

## Types d'appareils — Ordinateur
dropdown-bluetooth-type-computer = Ordinateur
dropdown-bluetooth-type-desktop = Ordinateur de bureau
dropdown-bluetooth-type-server = Serveur
dropdown-bluetooth-type-laptop = Ordinateur portable
dropdown-bluetooth-type-handheld = PC de poche
dropdown-bluetooth-type-palm = PC Palm
dropdown-bluetooth-type-wearable-computer = Ordinateur vestimentaire
dropdown-bluetooth-type-computer-tablet = Tablette

## Types d'appareils — Téléphone
dropdown-bluetooth-type-phone = Téléphone
dropdown-bluetooth-type-cellular = Cellulaire
dropdown-bluetooth-type-cordless = Sans fil
dropdown-bluetooth-type-smartphone = Téléphone intelligent
dropdown-bluetooth-type-modem = Modem

## Types d'appareils — Réseau
dropdown-bluetooth-type-network = Point d'accès

## Types d'appareils — Audio/Vidéo
dropdown-bluetooth-type-headset = Casque d'écoute
dropdown-bluetooth-type-handsfree = Mains libres
dropdown-bluetooth-type-microphone = Microphone
dropdown-bluetooth-type-loudspeaker = Haut-parleur
dropdown-bluetooth-type-headphones = Écouteurs
dropdown-bluetooth-type-portable-audio = Audio portable
dropdown-bluetooth-type-car-audio = Audio de voiture
dropdown-bluetooth-type-set-top-box = Boîtier décodeur
dropdown-bluetooth-type-hifi = Audio haute fidélité
dropdown-bluetooth-type-vcr = Magnétoscope
dropdown-bluetooth-type-video-camera = Caméra vidéo
dropdown-bluetooth-type-camcorder = Caméscope
dropdown-bluetooth-type-video-monitor = Moniteur vidéo
dropdown-bluetooth-type-video-display = Écran vidéo et haut-parleur
dropdown-bluetooth-type-video-conferencing = Vidéoconférence
dropdown-bluetooth-type-gaming = Jeu/Jouet
dropdown-bluetooth-type-audio-video = Audio/Vidéo

## Types d'appareils — Périphérique
dropdown-bluetooth-type-keyboard = Clavier
dropdown-bluetooth-type-mouse = Souris
dropdown-bluetooth-type-combo-keyboard = Clavier/Souris
dropdown-bluetooth-type-joystick = Manche de jeu
dropdown-bluetooth-type-gamepad = Manette
dropdown-bluetooth-type-remote = Télécommande
dropdown-bluetooth-type-sensing = Capteur
dropdown-bluetooth-type-tablet = Tablette graphique
dropdown-bluetooth-type-card-reader = Lecteur de cartes
dropdown-bluetooth-type-peripheral = Périphérique

## Types d'appareils — Imagerie
dropdown-bluetooth-type-imaging = Imagerie
dropdown-bluetooth-type-display = Écran
dropdown-bluetooth-type-camera = Appareil photo
dropdown-bluetooth-type-scanner = Numériseur
dropdown-bluetooth-type-printer = Imprimante

## Types d'appareils — Vestimentaire
dropdown-bluetooth-type-wearable = Accessoire connecté
dropdown-bluetooth-type-wrist-watch = Montre
dropdown-bluetooth-type-pager = Téléavertisseur
dropdown-bluetooth-type-jacket = Veste
dropdown-bluetooth-type-helmet = Casque
dropdown-bluetooth-type-glasses = Lunettes

## Types d'appareils — Jouet
dropdown-bluetooth-type-toy = Jouet
dropdown-bluetooth-type-robot = Robot
dropdown-bluetooth-type-vehicle = Véhicule
dropdown-bluetooth-type-doll = Poupée
dropdown-bluetooth-type-controller = Manette
dropdown-bluetooth-type-game = Jeu

## Types d'appareils — Autre
dropdown-bluetooth-type-health = Santé
dropdown-bluetooth-type-unknown = Appareil Bluetooth

## Noms de services (pour RequestServiceAuthorization)
dropdown-bluetooth-service-serial-port = Port série
dropdown-bluetooth-service-lan-access = Accès au réseau local
dropdown-bluetooth-service-dialup-networking = Réseau commuté
dropdown-bluetooth-service-object-push = Transfert d'objets
dropdown-bluetooth-service-file-transfer = Transfert de fichiers
dropdown-bluetooth-service-headset = Audio du casque
dropdown-bluetooth-service-audio-source = Source audio
dropdown-bluetooth-service-audio-sink = Récepteur audio
dropdown-bluetooth-service-remote-control = Télécommande
dropdown-bluetooth-service-audio-distribution = Diffusion audio
dropdown-bluetooth-service-handsfree = Mains libres
dropdown-bluetooth-service-network-access = Accès réseau
dropdown-bluetooth-service-input-device = Périphérique d'entrée
dropdown-bluetooth-service-sim-access = Accès SIM
dropdown-bluetooth-service-phonebook = Accès au répertoire
dropdown-bluetooth-service-messaging = Messagerie
dropdown-bluetooth-service-unknown = Service Bluetooth
dropdown-bluetooth-service-proprietary = Service Bluetooth

## États vides
dropdown-bluetooth-no-devices-title = Aucun appareil trouvé
dropdown-bluetooth-no-devices-description = Assurez-vous que votre appareil est en mode de jumelage
dropdown-bluetooth-off-title = Bluetooth désactivé
dropdown-bluetooth-off-description = Activez le Bluetooth pour connecter des appareils
dropdown-bluetooth-no-adapter-title = Aucun adaptateur Bluetooth
dropdown-bluetooth-no-adapter-description = Aucun adaptateur Bluetooth n'a été détecté
dropdown-bluetooth-no-nearby = Aucun autre appareil à proximité
dropdown-bluetooth-no-new = Aucun nouvel appareil trouvé

## Jumelage
# DisplayPinCode : libellé du NIP au-dessus du code
dropdown-bluetooth-pairing-enter-pin = Entrez ce NIP sur l'appareil
# DisplayPinCode : instruction sous le code
dropdown-bluetooth-pairing-type-on-device = Tapez le NIP sur l'appareil, puis appuyez sur Entrée
# RequestPasskey : l'utilisateur entre le NIP affiché sur l'autre appareil
dropdown-bluetooth-pairing-enter-shown-pin = Entrez le NIP affiché sur l'appareil
# RequestConfirmation : les deux côtés affichent le même code, l'utilisateur confirme
dropdown-bluetooth-pairing-confirm-code = Confirmez que ce code correspond à celui affiché sur l'appareil :
# DisplayPasskey : progression pendant que l'autre appareil tape le code
dropdown-bluetooth-pairing-entering = { $entered } sur { $total } chiffres entrés
# RequestAuthorization : jumelage simple, l'utilisateur accepte ou refuse
dropdown-bluetooth-pairing-allow-pairing = Autoriser le jumelage de cet appareil ?
# AuthorizeService : autorisation de service après le jumelage
dropdown-bluetooth-pairing-service-allow = Autoriser cet appareil à accéder au service demandé ?
# RequestPinCode : saisie de NIP classique
dropdown-bluetooth-pairing-enter-legacy-pin = Entrez le NIP de cet appareil
# Texte indicatif pour le champ de saisie du NIP
dropdown-bluetooth-pairing-pin-placeholder = NIP
dropdown-bluetooth-pairing-common-pins = NIP courants : 0000, 1234, 1111
# Échec du jumelage
dropdown-bluetooth-pairing-failed = Échec du jumelage. Le NIP est peut-être incorrect ou la demande a expiré.

### Menu déroulant du calendrier

dropdown-calendar-title = Calendrier

## Affichage de l'horloge

cal-day-sunday = dimanche
cal-day-monday = lundi
cal-day-tuesday = mardi
cal-day-wednesday = mercredi
cal-day-thursday = jeudi
cal-day-friday = vendredi
cal-day-saturday = samedi

# { $month } — nom du mois localisé, { $day } — numéro du jour, { $year } — année à quatre chiffres
cal-clock-date-rest = {" "}{ $day } { $month } { $year }

## Widget calendrier

cal-today = Aujourd'hui

# { $month } — nom du mois localisé, { $year } — année à quatre chiffres
cal-month-year = { $month } { $year }

cal-weekday-sun = Di
cal-weekday-mon = Lu
cal-weekday-tue = Ma
cal-weekday-wed = Me
cal-weekday-thu = Je
cal-weekday-fri = Ve
cal-weekday-sat = Sa

cal-month-january = janvier
cal-month-february = février
cal-month-march = mars
cal-month-april = avril
cal-month-may = mai
cal-month-june = juin
cal-month-july = juillet
cal-month-august = août
cal-month-september = septembre
cal-month-october = octobre
cal-month-november = novembre
cal-month-december = décembre

### Menu déroulant du tableau de bord

dropdown-dashboard-title = Tableau de bord
dropdown-dashboard-open-settings = Ouvrir les paramètres

## Actions rapides
dropdown-dashboard-wifi = Wi-Fi
dropdown-dashboard-bluetooth = Bluetooth
dropdown-dashboard-airplane = Mode avion
dropdown-dashboard-dnd = Ne pas déranger
dropdown-dashboard-idle-inhibit = Inhiber la veille
dropdown-dashboard-power-saver = Économie d'énergie

## Contrôles
dropdown-dashboard-volume = Volume
dropdown-dashboard-no-device = Aucun périphérique de sortie

## Média
dropdown-dashboard-now-playing = En cours de lecture
dropdown-dashboard-no-media-title = Aucun média
dropdown-dashboard-no-media-description = Aucun média en cours de lecture

## Batterie
dropdown-dashboard-battery = Batterie
dropdown-dashboard-battery-charging = En charge
dropdown-dashboard-battery-discharging = En décharge
dropdown-dashboard-battery-fully-charged = Complètement chargé
dropdown-dashboard-battery-pending-charge = Charge en attente
dropdown-dashboard-battery-pending-discharge = Décharge en attente
dropdown-dashboard-battery-empty = Vide
dropdown-dashboard-battery-unknown = Inconnu
dropdown-dashboard-battery-time-hm = ~{ $hours } h { $minutes } min
dropdown-dashboard-battery-time-m = ~{ $minutes } min
dropdown-dashboard-battery-profile-saver = Économie
dropdown-dashboard-battery-profile-balanced = Équilibré
dropdown-dashboard-battery-profile-performance = Performance

## Réseau
dropdown-dashboard-network = Réseau
dropdown-dashboard-network-disconnected = Déconnecté
dropdown-dashboard-network-connecting = Connexion…
dropdown-dashboard-network-ethernet = Ethernet
dropdown-dashboard-network-wifi-off = Wi-Fi désactivé
dropdown-dashboard-network-speed-kbs = Ko/s
dropdown-dashboard-network-speed-mbs = Mo/s

## Statistiques système
dropdown-dashboard-system = Système
dropdown-dashboard-cpu = Processeur
dropdown-dashboard-ram = Mémoire
dropdown-dashboard-disk = Disque
dropdown-dashboard-temp = Temp. proc.

## Session utilisateur
dropdown-dashboard-lock = Verrouiller
dropdown-dashboard-logout = Fermer la session
dropdown-dashboard-reboot = Redémarrer
dropdown-dashboard-power-off = Éteindre

### Menu déroulant du média

dropdown-media-title = En cours de lecture
dropdown-media-no-player-title = Aucun média en cours
dropdown-media-no-player-description = Lancez la lecture d'un média dans n'importe quelle application pour le contrôler ici
dropdown-media-sources = Sources de médias
dropdown-media-unknown-title = Titre inconnu
dropdown-media-unknown-artist = Artiste inconnu
dropdown-media-unknown-album = Album inconnu

### Menu déroulant du réseau

dropdown-network-title = Réseau
dropdown-network-active-connections = Connexions actives
dropdown-network-active-connection = Connexion active
dropdown-network-available = Réseaux disponibles
dropdown-network-connected = Connecté
dropdown-network-connecting = Connexion
dropdown-network-connect = Connecter
dropdown-network-disconnect = Déconnecter
dropdown-network-forget = Oublier
dropdown-network-dismiss = Fermer
dropdown-network-error = Erreur
dropdown-network-cancel = Annuler
dropdown-network-password-placeholder = Entrez le mot de passe
dropdown-network-ethernet = Ethernet
dropdown-network-wifi = Wi-Fi
dropdown-network-no-networks-title = Aucun réseau trouvé
dropdown-network-no-networks-description = Assurez-vous que le Wi-Fi est activé et relancez la recherche
dropdown-network-no-adapter-title = Aucun adaptateur Wi-Fi
dropdown-network-no-adapter-description = Aucun adaptateur sans fil n'a été détecté sur ce système

## Types de sécurité

dropdown-network-security-open = Ouvert
dropdown-network-security-wep = WEP
dropdown-network-security-wpa = WPA
dropdown-network-security-wpa2 = WPA2
dropdown-network-security-wpa3 = WPA3
dropdown-network-security-enterprise = Entreprise
dropdown-network-security-saved = { $security } · Enregistré

## Étapes de connexion

dropdown-network-step-preparing = Préparation…
dropdown-network-step-configuring = Configuration…
dropdown-network-step-authenticating = Authentification…
dropdown-network-step-obtaining-ip = Obtention de l'adresse IP…
dropdown-network-step-verifying = Vérification de la connexion…

## Erreurs de connexion

dropdown-network-error-wrong-password = Échec de l'authentification
dropdown-network-error-timeout = Délai de connexion dépassé
dropdown-network-error-ip-config = Impossible d'obtenir une adresse IP
dropdown-network-error-not-found = Réseau introuvable
dropdown-network-error-generic = Échec de la connexion

### Menu déroulant de la météo

dropdown-weather-title = Météo

## Statistiques
dropdown-weather-humidity = Humidité
dropdown-weather-wind = Vent
dropdown-weather-uv = Indice UV
dropdown-weather-rain = Pluie

## Sections
dropdown-weather-hourly = Prévisions horaires
dropdown-weather-daily = Prévisions sur 5 jours

## Heures du soleil
dropdown-weather-sunrise = Lever du soleil
dropdown-weather-sunset = Coucher du soleil

## Affichage du temps
# { $minutes } est le nombre de minutes depuis la dernière mise à jour
dropdown-weather-updated-ago = Mis à jour il y a { $minutes } min
dropdown-weather-today = Aujourd'hui
dropdown-weather-now = Maintenant

## Abréviations des jours
dropdown-weather-day-sun = dim.
dropdown-weather-day-mon = lun.
dropdown-weather-day-tue = mar.
dropdown-weather-day-wed = mer.
dropdown-weather-day-thu = jeu.
dropdown-weather-day-fri = ven.
dropdown-weather-day-sat = sam.

## Actions
dropdown-weather-refresh = Actualiser

## États
dropdown-weather-loading = Récupération des données météo…
dropdown-weather-error-title = Impossible de charger la météo
dropdown-weather-error-api-key = { $provider } nécessite une clé API.
dropdown-weather-error-location = L'emplacement « { $query } » est introuvable.
dropdown-weather-error-network = Impossible de joindre le service météo.
dropdown-weather-error-rate-limit = Trop de requêtes. Réessayez plus tard.
dropdown-weather-error-unknown = Une erreur est survenue. Réessayez plus tard.
dropdown-weather-retry = Réessayer

